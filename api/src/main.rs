#[macro_use]
extern crate rocket;

mod auth;
mod db;
mod message;
mod room;
mod user;

use auth::{validate_login, validate_user_key};
use db::{establish_connection, load_rooms};
use message::{add_message, load_messages, FormMessage};
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::sync::RwLock;
use rocket::{Shutdown, State};
use room::{add_room, add_user_room, select_users_room, FormAddRoom, FormAddUserRoom};
use std::collections::HashMap;
use user::{add_user, logout, user_select_id, FormAddUser};

type Convs = RwLock<HashMap<i64, Sender<String>>>;

#[post("/adduser", data = "<form>")]
fn post_adduser(form: Form<FormAddUser>) -> String {
    let conn = establish_connection().unwrap();
    let user = add_user(&conn, form.into_inner());
    match user {
        Ok(user) => format!(
            "{{ \"status_code\": {}, \"status\": \"Created\", \"user_id\": {}, \"username\": \"{}\", \"api_key\": \"{}\" }}",
            Status::Created.code, user.id, user.username, user.api_key
        ),
        Err(e) => format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            e
        ),
    }
}

#[get("/user/<user_id>")]
fn get_user(user_id: i64) -> String {
    let conn = establish_connection().unwrap();
    let user = user_select_id(&conn, user_id);
    match user {
        Ok(user) => {
            format!(
            "{{ \"status_code\": {}, \"status\": \"Ok\", \"user_id\": {}, \"username\": \"{}\" }}",
            Status::Ok.code, user.id, user.username
        )
        }
        Err(e) => format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            e
        ),
    }
}

#[post("/login", data = "<form>")]
fn post_login(form: Form<FormAddUser>) -> String {
    let conn = establish_connection().unwrap();
    let user = form.into_inner();

    match validate_login(&conn, user.username.as_str(), user.password.as_str()){
        Ok((id, api_key))=>
            format!(
                "{{ \"status_code\": {}, \"status\": \"Accepted\", \"user_id\": {}, \"api_key\": \"{}\" }}",
                Status::Accepted.code,
                id,
                api_key
            ),
        Err(r)=>
            format!(
                "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
                Status::Unauthorized.code,
                r
            )
    }
}

#[post("/room", data = "<form>")]
async fn post_addroom(form: Form<FormAddRoom>, convs: &State<Convs>) -> String {
    let conn = establish_connection().unwrap();

    let inform = form.into_inner();
    let user_id = inform.user_id;
    let user = validate_user_key(&conn, inform.user_id, inform.api_key.as_str());
    if user.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            user.unwrap_err()
        );
    }

    let room = add_room(&conn, inform).unwrap();

    let lock = convs.read().await;
    if let Some(conv) = lock.get(&user_id) {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.send(room.serialize());
    }

    return format!(
        "{{ \"status_code\": {}, \"status\": \"Created\", \"api_key\": \"{}\" }}",
        Status::Created.code,
        user.unwrap()
    );
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<user_id>?<api_key>")]
async fn get_events(
    user_id: i64,
    api_key: String,
    convs: &State<Convs>,
    mut end: Shutdown,
) -> Result<EventStream![], String> {
    let conn = establish_connection().unwrap();

    let bduser = user_select_id(&conn, user_id)?;
    let bdapi_key = bduser.api_key.as_str();

    if bdapi_key.eq("") {
        return Err(format!("bad user id or api key"));
    }

    if !bdapi_key.eq(api_key.as_str()) {
        return Err(format!("bad user id or api key"));
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

    let messages = load_messages(&conn, user_id).unwrap();
    let rooms = load_rooms(&conn, user_id).unwrap();

    return Ok(EventStream! {
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
                        let _ = logout(&conn, user_id).unwrap();
                        break;
                    },
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    });
}

#[post("/message", data = "<form>")]
async fn post_message(form: Form<FormMessage>, convs: &State<Convs>) -> String {
    let conn = establish_connection().unwrap();

    let inform = form.into_inner();
    let room_id = inform.room_id;
    let user = validate_user_key(&conn, inform.user_id, inform.api_key.as_str());
    if user.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            user.unwrap_err()
        );
    }

    let message = add_message(&conn, inform).unwrap();
    let smessage = message.serialize();

    let lock = convs.read().await;

    let users = select_users_room(&conn, room_id);
    if users.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"InternalServerError\", \"reason\": \"{}\" }}",
            Status::InternalServerError.code,
            users.unwrap_err()
        );
    }

    for user_id in users.unwrap() {
        if let Some(conv) = lock.get(&user_id) {
            // A send 'fails' if there are no active subscribers. That's okay.
            let _ = conv.send(smessage.to_string());
        }
    }

    return format!(
        "{{ \"status_code\": {}, \"status\": \"Created\", \"api_key\": \"{}\" }}",
        Status::Created.code,
        user.unwrap()
    );
}

#[post("/invite", data = "<form>")]
async fn post_invite(form: Form<FormAddUserRoom>, convs: &State<Convs>) -> String {
    let conn = establish_connection().unwrap();

    let inform = form.into_inner();
    let user = validate_user_key(&conn, inform.user_id, inform.api_key.as_str());
    if user.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            user.unwrap_err()
        );
    }

    let room = add_user_room(&conn, inform);

    if room.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"BadRequest\", \"reason\": \"{}\" }}",
            Status::BadRequest.code,
            room.unwrap_err()
        );
    }
    let room = room.unwrap();

    let lock = convs.read().await;
    if let Some(conv) = lock.get(&room.1) {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.send(room.0.serialize());
        let messages = load_messages(&conn, room.1).unwrap();
        for message in messages{
            let _ = conv.send(message.serialize());
        }
    }

    return format!(
        "{{ \"status_code\": {}, \"status\": \"Created\", \"api_key\": \"{}\" }}",
        Status::Created.code,
        user.unwrap()
    );
}

mod cors;

#[launch]
fn rocket() -> _ {
    let c: Convs = RwLock::new(HashMap::<i64, Sender<String>>::new());

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
