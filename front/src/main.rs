#![allow(non_snake_case)]

mod conv;
mod create_user;
mod event_source;
mod home;
mod login;
mod room;
mod side_bar;
mod structs;

pub const BASE_API_URL: &'static str = "http://127.0.0.1:8000";

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use room::Room;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use structs::Message;

use crate::conv::Conv;
use crate::create_user::CreateUser;
use crate::event_source::{event_receiver, EventReceiver, SourceState};
use crate::home::Home;
use crate::login::LogIn;
use crate::structs::User;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    LogIn {},
    #[route("/create-user")]
    CreateUser {},
    #[route("/:room_id/:room_name")]
    Conv { room_id: i64, room_name:String }
}

pub type AccountManager = Option<User>;

type Messages = Arc<Mutex<Box<HashMap<i64, Vec<Message>>>>>;
type Rooms = Arc<Mutex<Box<Vec<Room>>>>;

fn page(cx: Scope) -> Element {
    let _ = use_shared_state_provider::<Messages>(cx, || {
        Arc::new(Mutex::new(Box::new(HashMap::<i64, Vec<Message>>::new())))
    });
    let _ = use_shared_state_provider::<Rooms>(cx, || {
        Arc::new(Mutex::new(Box::new(Vec::<Room>::new())))
    });
    let _ = use_shared_state_provider::<SourceState>(cx, || SourceState::Disconnected);
    let _ = use_shared_state_provider::<AccountManager>(cx, || None);

    let messages = use_shared_state::<Messages>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();

    let message_sender: EventReceiver<Message> =
        event_receiver(cx, messages, |messages, message: Message| {
            let m = messages.write();
            let mut messages = m.lock().unwrap();

            if !messages.contains_key(&message.room_id) {
                messages.insert(message.room_id, Vec::new());
            }
            let vec = messages.get_mut(&message.room_id).unwrap();
            vec.push(message);
        });

    let room_sender: EventReceiver<Room> = event_receiver(cx, rooms, |rooms, room: Room| {
        rooms.write().lock().unwrap().push(room);
    });

    let source_state_sender: EventReceiver<SourceState> =
        event_receiver(cx, source_state, |source_state, state: SourceState| {
            *source_state.write() = state;
        });

    let r = user.read();
    if *source_state.read() == SourceState::Disconnected {
        if let Some(a) = r.as_ref() {
            let _ = event_source::MyEventSource::new(
                a.id,
                a.api_key.as_str(),
                &message_sender,
                &room_sender,
                &source_state_sender,
            );
        }
    }

    render! {
        link { rel: "stylesheet", href: "../dist/reset.css" }
        link { rel: "stylesheet", href: "../dist/style.css" }
        Router::<Route> {}
    }
}

fn main() {
    // launch the web app
    dioxus_desktop::launch(page);
}
