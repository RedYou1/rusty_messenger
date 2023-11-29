//! Point d'entré du programme de l'api.
//! 
//! Il impléments les routes de l'api.

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod test_event_source;
#[cfg(test)]
mod tests;

mod auth;
mod cors;
mod database;
mod date_time_sql;
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
enum ReponseJson {
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

/// Crée un utilisateur
#[post("/user", data = "<form>")]
fn post_user(form: Form<FormAddUser>) -> ReponseJson {
    let connection_bd = connection_bd();
    match connection_bd.ajout_user(form.into_inner()) {
        Ok(user) => ReponseJson::Created(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            user.user_id, user.api_key
        )),
        Err(_) => {
            ReponseJson::Unauthorized(String::from("{ \"reason\": \"Identifiant déjà pris\" }"))
        }
    }
}

/// Récupère le nom d'un utilisateur
#[get("/user/<user_id>")]
fn get_user(user_id: i64) -> ReponseJson {
    let connection_bd = connection_bd();
    match connection_bd.user_select_id(user_id) {
        Ok(user) => ReponseJson::Ok(format!(
            "{{ \"user_id\": {}, \"username\": \"{}\" }}",
            user.id, user.username
        )),
        Err(_) => ReponseJson::BadRequest(String::from("{ \"reason\": \"Mauvais id\" }")),
    }
}

/// Connecte l'utilisateur (crée une api_key)
#[post("/login", data = "<form>")]
fn post_login(form: Form<FormAddUser>) -> ReponseJson {
    let connection_bd = connection_bd();
    let form = form.into_inner();

    match connection_bd.connecter_utilisateur(form.username.as_str(), form.password.as_str()) {
        Ok(auth) => ReponseJson::Accepted(format!(
            "{{ \"user_id\": {}, \"api_key\": \"{}\" }}",
            auth.user_id, auth.api_key
        )),
        Err(_) => {
            ReponseJson::Unauthorized(String::from("{ \"reason\": \"Mauvais identifiant ou mot de passe\" }"))
        }
    }
}

/// Crée un salon
#[post("/room", data = "<form>")]
async fn post_room(form: Form<FormAddRoom>, convs: &State<EventStreams>) -> ReponseJson {
    let connection_bd = connection_bd();
    let form = form.into_inner();
    let user_id = form.user_id;

    let user = match connection_bd.verification_api_key_de_utilisateur(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ReponseJson::Unauthorized(String::from(
                "{ \"reason\": \"Mauvais id ou api key\" }",
            ));
        }
    };

    let room = connection_bd.ajout_room(form).unwrap();

    let lock = convs.read().await;
    if let Some(event_stream) = lock.get(&user_id) {
        event_stream.send(room.serialize()).unwrap();
    }

    ReponseJson::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

#[derive(Responder)]
enum Reponse<T> {
    #[response(status = 200)]
    Ok(T),
    #[response(status = 401, content_type = "json")]
    Unauthorized(String),
}

/// Crée l'Event Stream
#[get("/events/<user_id>?<api_key>")]
async fn get_events(
    user_id: i64,
    api_key: String,
    event_streams: &State<EventStreams>,
    mut end: Shutdown,
) -> Reponse<EventStream![]> {
    let connection_bd = connection_bd();

    let bd_user = match connection_bd.user_select_id(user_id) {
        Ok(bd_user) => bd_user,
        Err(_) => return Reponse::Unauthorized(String::from("Mauvais id ou api key")),
    };
    let bd_api_key = bd_user.api_key.as_str();

    if bd_api_key.eq("") || !bd_api_key.eq(api_key.as_str()) {
        return Reponse::Unauthorized(String::from("Mauvais id ou api key"));
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

    let messages = connection_bd.recupere_messages(user_id).unwrap();
    let rooms = connection_bd.recupere_rooms(user_id).unwrap();

    Reponse::Ok(EventStream! {
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
                        connection_bd.logout(user_id).unwrap();
                        break;
                    },
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            });
        }
    })
}

/// Envoie un message
#[post("/message", data = "<form>")]
async fn post_message(form: Form<FormMessage>, event_streams: &State<EventStreams>) -> ReponseJson {
    let connection_bd = connection_bd();
    let form = form.into_inner();
    let room_id = form.room_id;

    let user = match connection_bd.verification_api_key_de_utilisateur(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ReponseJson::Unauthorized(String::from(
                "{ \"reason\": \"Mauvais id ou api key\" }",
            ));
        }
    };

    let message = connection_bd.ajout_message(form).unwrap().serialize();
    let users = match connection_bd.select_users_room(room_id) {
        Ok(users) => users,
        Err(_) => {
            return ReponseJson::BadRequest(String::from("{ \"reason\": \"Ce salon n'exists pas\" }"))
        }
    };

    let lock = event_streams.read().await;
    for user_id in users {
        if let Some(event_stream) = lock.get(&user_id) {
            event_stream.send(message.to_string()).unwrap();
        }
    }

    ReponseJson::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

/// Invite un utilisateur dans un salon
#[post("/invite", data = "<form>")]
async fn post_invite(
    form: Form<FormAddUserRoom>,
    event_streams: &State<EventStreams>,
) -> ReponseJson {
    let connection_bd = connection_bd();
    let form = form.into_inner();
    let user = match connection_bd.verification_api_key_de_utilisateur(form.user_id, form.api_key.as_str()) {
        Ok(user) => user,
        Err(_) => {
            return ReponseJson::Unauthorized(String::from(
                "{ \"reason\": \"Mauvais id ou api key\" }",
            ));
        }
    };

    let room = match connection_bd.ajout_user_room(form) {
        Ok(room) => room,
        Err(e) => {
            return ReponseJson::BadRequest(format!(
                "{{ \"api_key\": \"{}\", \"reason\": \"{}\" }}",
                user, e
            ));
        }
    };

    let lock = event_streams.read().await;
    if let Some(event_stream) = lock.get(&room.1) {
        event_stream.send(room.0.serialize()).unwrap();
        for message in connection_bd.recupere_messages(room.1).unwrap() {
            event_stream.send(message.serialize()).unwrap();
        }
    }

    ReponseJson::Created(format!("{{ \"api_key\": \"{}\" }}", user))
}

static mut IS_UNIT_TEST: bool = false;
fn connection_bd() -> Database {
    Database::new(unsafe { IS_UNIT_TEST }).unwrap()
}

pub fn build(is_unit_test: bool) -> Rocket<Build> {
    let c: EventStreams = RwLock::new(HashMap::<i64, Sender<String>>::new());
    unsafe {
        IS_UNIT_TEST = is_unit_test;
    }
    connection_bd().cree_tables().unwrap();

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
