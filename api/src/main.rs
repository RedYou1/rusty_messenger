#[macro_use]
extern crate rocket;

#[cfg(test)]
mod test_event_stream;
#[cfg(test)]
mod tests;

mod auth;
mod cors;
mod db;
mod message;
mod room;
mod user;

use db::MyConnection;
use message::FormMessage;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::response::Responder;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::sync::RwLock;
use rocket::{Build, Rocket, Shutdown, State};
use room::{FormAddRoom, FormAddUserRoom};
use std::collections::HashMap;
use user::FormAddUser;

type Convs = RwLock<HashMap<i64, Sender<String>>>;

#[derive(Debug, Responder)]
enum ApiResponse {
    #[response(status = 200, content_type = "json")]
    Ok(String),
    #[response(status = 201, content_type = "json")]
    Created(String),
    #[response(status = 202, content_type = "json")]
    Accepted(String),
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 401, content_type = "json")]
    Unauthorized(String),
    #[response(status = 500, content_type = "json")]
    InternalServerError(String),
}

#[post("/adduser", data = "<form>")]
fn post_adduser(form: Form<FormAddUser>) -> ApiResponse {
    let conn = connection();
    match conn.add_user(form.into_inner()) {
        Ok(user) => ApiResponse::Created(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            user.user_id, user.api_key
        )),
        Err(_) => ApiResponse::Unauthorized(format!(
            "{{ \"reason\": \"{}\" }}",
            "Username Already Taken"
        )),
    }
}

#[get("/user/<user_id>")]
fn get_user(user_id: i64) -> ApiResponse {
    let conn = connection();
    match conn.user_select_id(user_id) {
        Ok(user) => ApiResponse::Ok(format!(
            "{{ \"user_id\": {}, \"username\": \"{}\" }}",
            user.id, user.username
        )),
        Err(e) => ApiResponse::Unauthorized(format!("{{ \"reason\": \"{}\" }}", e)),
    }
}

#[post("/login", data = "<form>")]
fn post_login(form: Form<FormAddUser>) -> ApiResponse {
    let conn = connection();
    let user = form.into_inner();

    match conn.validate_login(user.username.as_str(), user.password.as_str()) {
        Ok((id, api_key)) => ApiResponse::Accepted(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            id, api_key
        )),
        Err(r) => ApiResponse::Unauthorized(format!("{{ \"reason\": \"{}\" }}", r)),
    }
}

#[post("/room", data = "<form>")]
async fn post_addroom(form: Form<FormAddRoom>, convs: &State<Convs>) -> ApiResponse {
    let conn = connection();

    let inform = form.into_inner();
    let user_id = inform.user_id;
    let user = match conn.validate_user_key(inform.user_id, inform.api_key.as_str()) {
        Ok(user) => user,
        Err(e) => {
            return ApiResponse::Unauthorized(format!("{{ \"reason\": \"{}\" }}", e));
        }
    };

    let room = conn.add_room(inform).unwrap();

    let lock = convs.read().await;
    if let Some(conv) = lock.get(&user_id) {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.send(room.serialize());
    }

    ApiResponse::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

#[derive(Responder)]
enum ApiResponseEvents<T> {
    #[response(status = 200)]
    Ok(T),
    #[response(status = 401)]
    Unauthorized(String),
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<user_id>?<api_key>")]
async fn get_events(
    user_id: i64,
    api_key: String,
    convs: &State<Convs>,
    mut end: Shutdown,
) -> ApiResponseEvents<EventStream![]> {
    let conn = connection();

    let bduser = match conn.user_select_id(user_id) {
        Ok(bduser) => bduser,
        Err(_) => return ApiResponseEvents::Unauthorized("bad user id or api key".to_string()),
    };
    let bdapi_key = bduser.api_key.as_str();

    if bdapi_key.eq("") {
        return ApiResponseEvents::Unauthorized("bad user id or api key".to_string());
    }

    if !bdapi_key.eq(api_key.as_str()) {
        return ApiResponseEvents::Unauthorized("bad user id or api key".to_string());
    }

    let mut rx;

    {
        let mut trx = None;
        {
            let lock = convs.read().await;
            if let Some(conv) = lock.get(&user_id) {
                trx = Some(conv.subscribe());
            }
        }
        match trx {
            Some(t) => rx = t,
            None => {
                let t = Some(channel::<String>(1024).0);
                let mut lock = convs.write().await;
                lock.insert(user_id, t.unwrap());
                rx = lock.get(&user_id).unwrap().subscribe();
            }
        }
    }

    let messages = conn.load_messages(user_id).unwrap();
    let rooms = conn.load_rooms(user_id).unwrap();

    ApiResponseEvents::Ok(EventStream! {
        for rm in rooms {
            yield Event::json(&rm.serialize());
        };
        for msg in messages {
            yield Event::json(&msg.serialize());
        };
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => {
                        let _ = conn.logout(user_id).unwrap();
                        break;
                    },
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    })
}

#[post("/message", data = "<form>")]
async fn post_message(form: Form<FormMessage>, convs: &State<Convs>) -> ApiResponse {
    let conn = connection();

    let inform = form.into_inner();
    let room_id = inform.room_id;
    let user = match conn.validate_user_key(inform.user_id, inform.api_key.as_str()) {
        Ok(user) => user,
        Err(e) => {
            return ApiResponse::Unauthorized(format!("{{ \"reason\": \"{}\" }}", e));
        }
    };

    let message = conn.add_message(inform).unwrap();
    let smessage = message.serialize();

    let lock = convs.read().await;

    let users = match conn.select_users_room(room_id) {
        Ok(users) => users,
        Err(e) => {
            return ApiResponse::InternalServerError(format!("{{ \"reason\": \"{}\" }}", e));
        }
    };

    for user_id in users {
        if let Some(conv) = lock.get(&user_id) {
            // A send 'fails' if there are no active subscribers. That's okay.
            let _ = conv.send(smessage.to_string());
        }
    }

    ApiResponse::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

#[post("/invite", data = "<form>")]
async fn post_invite(form: Form<FormAddUserRoom>, convs: &State<Convs>) -> ApiResponse {
    let conn = connection();

    let inform = form.into_inner();
    let user = match conn.validate_user_key(inform.user_id, inform.api_key.as_str()) {
        Ok(user) => user,
        Err(e) => {
            return ApiResponse::Unauthorized(format!("{{ \"reason\": \"{}\" }}", e));
        }
    };

    let room = match conn.add_user_room(inform) {
        Ok(room) => room,
        Err(e) => {
            return ApiResponse::BadRequest(format!(
                "{{ \"api_key\": \"{}\", \"reason\": \"{}\" }}",
                user, e
            ));
        }
    };

    let lock = convs.read().await;
    if let Some(conv) = lock.get(&room.1) {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.send(room.0.serialize());
        let messages = conn.load_messages(room.1).unwrap();
        for message in messages {
            let _ = conv.send(message.serialize());
        }
    }

    ApiResponse::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

static mut TEST: bool = false;
fn connection() -> MyConnection {
    unsafe { MyConnection::new(TEST).unwrap() }
}

pub fn build(test: bool) -> Rocket<Build> {
    let c: Convs = RwLock::new(HashMap::<i64, Sender<String>>::new());
    unsafe {
        TEST = test;
    }
    connection().ensure_tables().unwrap();

    rocket::build()
        .attach(crate::cors::CORS)
        .manage(c)
        .mount(
            "/",
            routes![
                post_adduser,
                post_login,
                get_events,
                get_user,
                post_message,
                post_addroom,
                post_invite
            ],
        )
        .mount("/", FileServer::from(relative!("static")))
}

#[launch]
fn rocket() -> _ {
    build(false)
}
