#![allow(non_snake_case)]

mod async_state;
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

use crate::async_state::AsyncStateSetter;
use crate::conv::Conv;
use crate::create_user::CreateUser;
use crate::event_source::MyEventSource;
use crate::event_source::SourceState;
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
    Conv { room_id: i64, room_name:String },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

pub type AccountManager = Arc<Mutex<Option<User>>>;

type Messages = Arc<Mutex<HashMap<i64, Vec<Message>>>>;
type Rooms = Arc<Mutex<Vec<Room>>>;
type Users = Arc<Mutex<HashMap<i64, String>>>;

fn page(cx: Scope) -> Element {
    let _ = use_shared_state_provider::<Messages>(cx, || {
        Arc::new(Mutex::new(HashMap::<i64, Vec<Message>>::new()))
    });
    let _ = use_shared_state_provider::<Rooms>(cx, || Arc::new(Mutex::new(Vec::<Room>::new())));
    let _ = use_shared_state_provider::<Users>(cx, || {
        Arc::new(Mutex::new(HashMap::<i64, String>::new()))
    });
    let _ = use_shared_state_provider::<SourceState>(cx, || SourceState::Disconnected);
    let _ = use_shared_state_provider::<AccountManager>(cx, || Arc::new(Mutex::new(None)));

    let messages = use_shared_state::<Messages>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();
    let event_source = use_state::<Option<MyEventSource>>(cx, || None);

    let message_sender = AsyncStateSetter::<Message>::new(cx, messages, |messages, message| {
        let m = messages.write();
        let mut messages = m.lock().unwrap();

        if !messages.contains_key(&message.room_id) {
            messages.insert(message.room_id, Vec::new());
        }
        let vec = messages.get_mut(&message.room_id).unwrap();
        vec.push(message);
    });

    let room_sender = AsyncStateSetter::<Room>::new(cx, rooms, |rooms, room| {
        rooms.write().lock().unwrap().push(room)
    });

    let source_state_sender =
        AsyncStateSetter::<SourceState>::new(cx, source_state, |source_state, state| {
            *source_state.write() = state
        });

    if *source_state.read() == SourceState::Disconnected {
        if let Some(a) = user.read().lock().unwrap().as_ref() {
            if event_source.is_some() {
                event_source.as_ref().unwrap().close();
            }
            event_source.set(Some(MyEventSource::new(
                a.id,
                a.api_key.as_str(),
                &message_sender,
                &room_sender,
                &source_state_sender,
            )));
        }
    }

    render! {
        link{ rel: "stylesheet", href: "/reset.css" }
        link{ rel: "stylesheet", href: "/style.css" }
        Router::<Route> {}
    }
}

#[inline_props]
fn PageNotFound(cx: Scope, route: Vec<String>) -> Element {
    render! {
        h1{ "404. Route: {route:?}, Not Found. :(" }
    }
}

fn main() {
    // launch the web app
    dioxus_web::launch(page);
}
