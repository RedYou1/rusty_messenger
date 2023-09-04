#[macro_use]
extern crate rocket;

mod auth;
mod db;
mod structs;

use std::collections::HashMap;

use auth::{validate_login, validate_user_key};
use db::user_select_id;
use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::sync::RwLock;
use rocket::{Shutdown, State};
use structs::{FormAddUser, FormMessage, MessageSerialized};

type Convs = RwLock<HashMap<i64, Sender<MessageSerialized>>>;

#[post("/adduser", data = "<form>")]
fn adduser(form: Form<FormAddUser>) -> String {
    let conn = db::establish_connection().unwrap();
    let user = db::add_user(&conn, form.into_inner()).unwrap();

    return format!(
        "{{ \"id\": {}, \"username\": \"{}\" }}",
        user.id, user.username
    );
}

#[post("/login", data = "<form>")]
fn login(form: Form<FormAddUser>) -> String {
    let conn = db::establish_connection().unwrap();
    let user = form.into_inner();
    let r = validate_login(&conn, user.username.as_str(), user.password.as_str());

    if r.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            r.unwrap_err()
        );
    }

    let (id, api_key) = r.unwrap();

    return format!(
        "{{ \"status_code\": {}, \"status\": \"Accepted\", \"user_id\": {}, \"api_key\": \"{}\" }}",
        Status::Accepted.code,
        id,
        api_key
    );
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<user_id>?<api_key>")]
async fn events(
    user_id: i64,
    api_key: String,
    convs: &State<Convs>,
    mut end: Shutdown,
) -> Result<EventStream![], String> {
    let conn = db::establish_connection().unwrap();

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
            let conv = lock.get(&user_id);
            if conv.is_some() {
                trx = Some(conv.unwrap().subscribe());
            }
        }
        match trx {
            Some(t) => rx = t,
            None => {
                let t = Some(channel::<MessageSerialized>(1024).0);
                let mut lock = convs.write().await;
                lock.insert(user_id, t.unwrap());
                rx = lock.get(&user_id).unwrap().subscribe();
            }
        }
    }

    let messages = db::load_messages(&conn, user_id).unwrap();

    return Ok(EventStream! {
        for msg in messages {
            yield Event::json(&msg.serialize());
        };
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => {
                        let _ = db::logout(&conn, user_id).unwrap();
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
async fn message(form: Form<FormMessage>, convs: &State<Convs>) -> String {
    let conn = db::establish_connection().unwrap();

    let inform = form.into_inner();
    let user = validate_user_key(&conn, inform.user_id, inform.api_key.as_str());
    if user.is_err() {
        return format!(
            "{{ \"status_code\": {}, \"status\": \"Unauthorized\", \"reason\": \"{}\" }}",
            Status::Unauthorized.code,
            user.unwrap_err()
        );
    }

    let message = db::add_message(&conn, inform).unwrap();

    let lock = convs.read().await;
    let conv = lock.get(&message.user_id);

    if conv.is_some() {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.unwrap().send(message.serialize());
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
    let c: Convs = RwLock::new(HashMap::<i64, Sender<MessageSerialized>>::new());

    rocket::build()
        .attach(crate::cors::CORS)
        .manage(c)
        .mount("/", routes![adduser, login, events, message])
        .mount("/", FileServer::from(relative!("static")))
}
