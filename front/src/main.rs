#![allow(non_snake_case)]

mod account_manager;
mod async_state;
mod messages;
mod create_user;
mod event_source;
mod home;
mod login;
mod room;
mod side_bar;
mod structs;

pub const BASE_API_URL: &'static str = "http://192.168.137.1:8000";

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use lib::{Message, Room};
use room::{OpRoomId, RoomData};
use std::collections::HashMap;

use crate::account_manager::AccountManager;
use crate::async_state::AsyncStateSetter;
use crate::messages::Conv;
use crate::create_user::CreateUser;
use crate::event_source::SourceState;
use crate::home::Home;
use crate::login::LogIn;
use crate::side_bar::SideBar;

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
    #[route("/rooms?:room_id")]
    SideBar { room_id: OpRoomId },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

pub struct Rooms(HashMap<i64, RoomData>);
pub struct Users(HashMap<i64, Option<String>>);

fn window(cx: Scope) -> Element {
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

    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();
    if account_manager.read().current_user().is_some() {
        match *source_state.read() {
            SourceState::Error => account_manager.write().retry_connection(),
            SourceState::Connected => account_manager.write_silent().set_connected(),
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
    dioxus_web::launch(window);
}
