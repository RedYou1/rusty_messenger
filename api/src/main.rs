#[macro_use]
extern crate rocket;

#[cfg(test)]
mod test_event_source;
#[cfg(test)]
mod tests;

mod auth;
mod cors;
mod database;
mod message;
mod room;
mod user;

use database::Database;
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

type EventStreams = RwLock<HashMap<i64, Sender<String>>>;

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
}

#[post("/user", data = "<form>")]
fn post_user(form: Form<FormAddUser>) -> ApiResponse {
    let bd_connection = connection();
    match bd_connection.add_user(form.into_inner()) {
        Ok(user) => ApiResponse::Created(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            user.user_id, user.api_key
        )),
        Err(_) => {
            ApiResponse::Unauthorized(String::from("{ \"reason\": \"Username Already Taken\" }"))
        }
    }
}

#[get("/user/<user_id>")]
fn get_user(user_id: i64) -> ApiResponse {
    let bd_connection = connection();
    match bd_connection.user_select_id(user_id) {
        Ok(user) => ApiResponse::Ok(format!(
            "{{ \"user_id\": {}, \"username\": \"{}\" }}",
            user.id, user.username
        )),
        Err(_) => ApiResponse::BadRequest(String::from("{ \"reason\": \"bad user id\" }")),
    }
}

#[post("/login", data = "<form>")]
fn post_login(form: Form<FormAddUser>) -> ApiResponse {
    let bd_connection = connection();
    let form = form.into_inner();

    match bd_connection.validate_login(form.username.as_str(), form.password.as_str()) {
        Ok(auth) => ApiResponse::Accepted(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            auth.user_id, auth.api_key
        )),
        Err(_) => {
            ApiResponse::Unauthorized(String::from("{ \"reason\": \"bad username or password\" }"))
        }
    }
}

#[post("/room", data = "<form>")]
async fn post_room(form: Form<FormAddRoom>, convs: &State<EventStreams>) -> ApiResponse {
    let bd_connection = connection();
    let form = form.into_inner();
    let user_id = form.user_id;

    let user = match bd_connection.validate_user_with_api_key(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ApiResponse::Unauthorized(String::from(
                "{ \"reason\": \"bad user id or api key\" }",
            ));
        }
    };

    let room = bd_connection.add_room(form).unwrap();

    let lock = convs.read().await;
    if let Some(event_stream) = lock.get(&user_id) {
        event_stream.send(room.serialize()).unwrap();
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

#[get("/events/<user_id>?<api_key>")]
async fn get_events(
    user_id: i64,
    api_key: String,
    event_streams: &State<EventStreams>,
    mut end: Shutdown,
) -> ApiResponseEvents<EventStream![]> {
    let bd_connection = connection();

    let bd_user = match bd_connection.user_select_id(user_id) {
        Ok(bd_user) => bd_user,
        Err(_) => return ApiResponseEvents::Unauthorized(String::from("bad user id or api key")),
    };
    let bd_api_key = bd_user.api_key.as_str();

    if bd_api_key.eq("") || !bd_api_key.eq(api_key.as_str()) {
        return ApiResponseEvents::Unauthorized(String::from("bad user id or api key"));
    }

    let mut event_receiver = match {
        let lock = event_streams.read().await;
        lock.get(&user_id)
            .map(|event_sender| event_sender.subscribe())
    } {
        Some(event_receiver) => event_receiver,
        None => {
            let event_sender = Some(channel::<String>(1024).0).unwrap();
            let mut lock = event_streams.write().await;
            lock.insert(user_id, event_sender);

            lock.get(&user_id).unwrap().subscribe()
        }
    };

    let messages = bd_connection.load_messages(user_id).unwrap();
    let rooms = bd_connection.load_rooms(user_id).unwrap();

    ApiResponseEvents::Ok(EventStream! {
        for room in rooms {
            yield Event::data(room.serialize());
        };
        for message in messages {
            yield Event::data(message.serialize());
        };
        loop {
            yield Event::data(select! {
                message = event_receiver.recv() => match message {
                    Ok(message) => message,
                    Err(RecvError::Closed) => {
                        bd_connection.logout(user_id).unwrap();
                        break;
                    },
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            });
        }
    })
}

#[post("/message", data = "<form>")]
async fn post_message(form: Form<FormMessage>, event_streams: &State<EventStreams>) -> ApiResponse {
    let bd_connection = connection();
    let form = form.into_inner();
    let room_id = form.room_id;

    let user = match bd_connection.validate_user_with_api_key(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ApiResponse::Unauthorized(String::from(
                "{ \"reason\": \"bad user id or api key\" }",
            ));
        }
    };

    let message = bd_connection.add_message(form).unwrap().serialize();
    let users = match bd_connection.select_users_room(room_id) {
        Ok(users) => users,
        Err(_) => {
            return ApiResponse::BadRequest(String::from("{ \"reason\": \"room doesnt exists\" }"))
        }
    };

    let lock = event_streams.read().await;
    for user_id in users {
        if let Some(event_stream) = lock.get(&user_id) {
            event_stream.send(message.to_string()).unwrap();
        }
    }

    ApiResponse::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

#[post("/invite", data = "<form>")]
async fn post_invite(
    form: Form<FormAddUserRoom>,
    event_streams: &State<EventStreams>,
) -> ApiResponse {
    let bd_connection = connection();
    let form = form.into_inner();
    let user = match bd_connection.validate_user_with_api_key(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ApiResponse::Unauthorized(String::from(
                "{ \"reason\": \"bad user id or api key\" }",
            ));
        }
    };

    let room = match bd_connection.add_user_room(form) {
        Ok(room) => room,
        Err(e) => {
            return ApiResponse::BadRequest(format!(
                "{{ \"api_key\": \"{}\", \"reason\": \"{}\" }}",
                user, e
            ));
        }
    };

    let lock = event_streams.read().await;
    if let Some(event_stream) = lock.get(&room.1) {
        event_stream.send(room.0.serialize()).unwrap();
        for message in bd_connection.load_messages(room.1).unwrap() {
            event_stream.send(message.serialize()).unwrap();
        }
    }

    ApiResponse::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

static mut IS_UNIT_TEST: bool = false;
fn connection() -> Database {
    unsafe { Database::new(IS_UNIT_TEST).unwrap() }
}

pub fn build(is_unit_test: bool) -> Rocket<Build> {
    let c: EventStreams = RwLock::new(HashMap::<i64, Sender<String>>::new());
    unsafe {
        IS_UNIT_TEST = is_unit_test;
    }
    connection().create_tables().unwrap();

    rocket::build()
        .attach(crate::cors::CORS)
        .manage(c)
        .mount(
            "/",
            routes![
                post_user,
                post_login,
                get_events,
                get_user,
                post_message,
                post_room,
                post_invite
            ],
        )
        .mount("/", FileServer::from(relative!("static")))
}

#[launch]
fn rocket() -> _ {
    build(false)
}
