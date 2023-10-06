#![allow(non_snake_case)]

mod account_manager;
mod async_state;
mod conv;
mod create_user;
mod event_source;
mod home;
mod login;
mod room;
mod side_bar;
mod structs;

pub const BASE_API_URL: &'static str = "http://172.19.67.102:8000";

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use lib::{Message, Room};
use room::RoomData;
use std::collections::HashMap;

use crate::account_manager::AccountManager;
use crate::async_state::AsyncStateSetter;
use crate::conv::Conv;
use crate::create_user::CreateUser;
use crate::event_source::SourceState;
use crate::home::Home;
use crate::login::LogIn;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    LogIn {},
    #[route("/create-user")]
    CreateUser {},
    #[route("/:room_id")]
    Conv { room_id: i64 },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

pub struct Rooms(HashMap<i64, RoomData>);
pub struct Users(HashMap<i64, Option<String>>);

fn page(cx: Scope) -> Element {
    let _ = use_shared_state_provider::<Rooms>(cx, || Rooms {
        0: HashMap::<i64, RoomData>::new(),
    });
    let _ = use_shared_state_provider::<Users>(cx, || Users {
        0: HashMap::<i64, Option<String>>::new(),
    });
    let _ = use_shared_state_provider::<SourceState>(cx, || SourceState::Error);

    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();

    let message_sender = AsyncStateSetter::<Message>::new(cx, rooms, |rooms, message| {
        let mut rooms = rooms.write();

        match rooms.0.get_mut(&message.room_id) {
            None => panic!("message add room_id:{} doesn't exists", message.room_id),
            Some(room) => room.messages.push(message),
        }
    });

    let room_sender = AsyncStateSetter::<Room>::new(cx, rooms, |rooms, room| {
        rooms.write().0.insert(
            room.id,
            RoomData {
                name: room.name,
                messages: Vec::new(),
            },
        );
    });

    let source_state_sender =
        AsyncStateSetter::<SourceState>::new(cx, source_state, |source_state, state| {
            *source_state.write() = state
        });
    let _ = use_shared_state_provider::<AccountManager>(cx, move || {
        AccountManager::new(message_sender, room_sender, source_state_sender)
    });

    let user = use_shared_state::<AccountManager>(cx).unwrap();
    if user.read().user().is_some() {
        match *source_state.read() {
            SourceState::Error => user.write().retry(),
            SourceState::Connected => user.write_silent().connected(),
            _ => {}
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
