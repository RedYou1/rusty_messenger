#![allow(non_snake_case)]

mod account;
mod conv;
mod event_source;
mod home;
mod login;
mod side_bar;
mod structs;

pub const BASE_API_URL: &'static str = "http://127.0.0.1:8000";

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use futures_channel::mpsc::UnboundedReceiver;
use futures_lite::stream::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use structs::Message;

use crate::account::AccountManager;
use crate::conv::Conv;
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
    #[route("/:room")]
    Conv { room: i64 }
}

type Messages = Arc<Mutex<Box<HashMap<i64, Vec<Message>>>>>;

fn page(cx: Scope) -> Element {
    let _ = use_shared_state_provider::<Messages>(cx, || {
        Arc::new(Mutex::new(Box::new(HashMap::<i64, Vec<Message>>::new())))
    });
    let _ = use_shared_state_provider::<SourceState>(cx, || SourceState::Disconnected);
    let _ = use_shared_state_provider::<AccountManager>(cx, || None);

    let messages = use_shared_state::<Messages>(cx).unwrap();
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();

    let message_sender = Arc::new(Mutex::new(
        use_coroutine(cx, |mut receiver: UnboundedReceiver<Message>| unsafe {
            let messages = messages as *const UseSharedState<Messages>;
            async move {
                loop {
                    match receiver.next().await {
                        Some(message) => {
                            {
                                let m = messages.as_ref().unwrap().write_silent();
                                let mut messages = m.lock().unwrap();

                                if !messages.contains_key(&message.room) {
                                    messages.insert(message.room, Vec::new());
                                }
                                let vec = messages.get_mut(&message.room).unwrap();
                                vec.push(message);
                            }
                            messages.as_ref().unwrap().write();
                        }
                        None => println!("None"),
                    }
                }
            }
        })
        .to_owned(),
    ));

    let source_state_sender = Arc::new(Mutex::new(
        use_coroutine(cx, |mut receiver: UnboundedReceiver<SourceState>| unsafe {
            let source_state = source_state as *const UseSharedState<SourceState>;
            async move {
                loop {
                    match receiver.next().await {
                        Some(state) => {
                            let mut s = source_state.as_ref().unwrap().write();
                            *s = state;
                        }
                        None => println!("None"),
                    }
                }
            }
        })
        .to_owned(),
    ));

    let r = user.read();
    if *source_state.read() == SourceState::Disconnected {
        if let Some(a) = r.as_ref() {
            let _ = event_source::MyEventSource::new(
                a.user.id,
                a.user.api_key.as_str(),
                &message_sender,
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
