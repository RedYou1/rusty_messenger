#[macro_use]
extern crate rocket;

mod db;
mod structs;

use std::collections::HashMap;

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::sync::RwLock;
use rocket::{Shutdown, State};
use structs::{FormMessage, MessageSerialized};

type Convs = RwLock<HashMap<usize, Sender<MessageSerialized>>>;

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<user_id>")]
async fn events(user_id: usize, convs: &State<Convs>, mut end: Shutdown) -> EventStream![] {
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

    let conn = db::establish_connection().unwrap();
    let messages = db::load_messages(&conn, user_id).unwrap();

    EventStream! {
        for msg in messages {
            yield Event::json(&msg.serialize());
        };
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

/// Receive a message from a form submission and broadcast it to any receivers.
#[post("/message", data = "<form>")]
async fn message(form: Form<FormMessage>, convs: &State<Convs>) {
    let conn = db::establish_connection().unwrap();
    let message = db::add_message(&conn, form.into_inner()).unwrap();

    let lock = convs.read().await;
    let conv = lock.get(&message.user_id);

    if conv.is_some() {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.unwrap().send(message.serialize());
    }
}

mod cors;

#[launch]
fn rocket() -> _ {
    let c: Convs = RwLock::new(HashMap::<usize, Sender<MessageSerialized>>::new());

    rocket::build()
        .attach(crate::cors::CORS)
        .manage(c)
        .mount("/", routes![message, events])
        .mount("/", FileServer::from(relative!("static")))
}
