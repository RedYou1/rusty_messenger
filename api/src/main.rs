#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::tokio::sync::RwLock;
use rocket::{Shutdown, State};

type Convs = RwLock<HashMap<u32, Sender<Message>>>;

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    pub room: u32,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events/<id>")]
async fn events(id: u32, convs: &State<Convs>, mut end: Shutdown) -> EventStream![] {
    let mut rx;

    {
        let mut trx = None;
        {
            let lock = convs.read().await;
            let conv = lock.get(&id);
            if conv.is_some() {
                trx = Some(conv.unwrap().subscribe());
            }
        }
        match trx {
            Some(t) => rx = t,
            None => {
                let t = Some(channel::<Message>(1024).0);
                let mut lock = convs.write().await;
                lock.insert(id, t.unwrap());
                rx = lock.get(&id).unwrap().subscribe();
            }
        }
    }

    EventStream! {
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
async fn message(form: Form<Message>, convs: &State<Convs>) {
    let message = form.into_inner();
    let lock = convs.read().await;
    let conv = lock.get(&message.room);

    if conv.is_some() {
        // A send 'fails' if there are no active subscribers. That's okay.
        let _ = conv.unwrap().send(message);
    }
}

mod cors;

#[launch]
fn rocket() -> _ {
    let c: Convs = RwLock::new(HashMap::<u32, Sender<Message>>::new());

    rocket::build()
        .attach(crate::cors::CORS)
        .manage(c)
        .mount("/", routes![message, events])
        .mount("/", FileServer::from(relative!("static")))
}
