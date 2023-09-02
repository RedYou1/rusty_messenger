#![allow(non_snake_case)]

mod structs;
const BASE_API_URL: &str = "http://127.0.0.1:8000";

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use futures_channel::mpsc::UnboundedReceiver;
use futures_lite::stream::StreamExt;
use sse_client::EventSource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use structs::{serialize, Message};
use tokio::runtime::Runtime;

use crate::structs::deserialize;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/:room")]
    Conv { room: usize }
}

static mut NOT_INITIALIZED: bool = true;

type Messages = Arc<Mutex<Box<HashMap<usize, Vec<Message>>>>>;

fn page(cx: Scope) -> Element {
    let _ = use_shared_state_provider::<Messages>(cx, || {
        Arc::new(Mutex::new(Box::new(HashMap::<usize, Vec<Message>>::new())))
    });
    let messages = use_shared_state::<Messages>(cx).unwrap();

    let sender = Arc::new(Mutex::new(
        use_coroutine(cx, |mut receiver: UnboundedReceiver<Message>| unsafe {
            let messages = messages as *const UseSharedState<Messages>;
            async move {
                loop {
                    match receiver.next().await {
                        Some(message) => {
                            {
                                let m = messages.as_ref().unwrap().write_silent();
                                let mut messages = m.lock().unwrap();

                                println!("inserting {}", message.date.to_string());
                                if !messages.contains_key(&message.room) {
                                    messages.insert(message.room, Vec::new());
                                }
                                let vec = messages.get_mut(&message.room).unwrap();
                                let rid = message.room;
                                vec.push(message);
                                println!("vec {} is now {}", rid, vec.len());
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

    unsafe {
        if NOT_INITIALIZED {
            let url = format!("{BASE_API_URL}/events/0");
            let event_source = EventSource::new(url.as_str()).unwrap();
            println!("new event_source");
            let sender_thread = Arc::clone(&sender);
            event_source.on_message(move |event| {
                let value = json::parse(event.data.as_str()).unwrap();
                let message = deserialize(
                    value["date"].as_i64().unwrap(),
                    value["room"].as_usize().unwrap(),
                    value["user_id"].as_usize().unwrap(),
                    value["text"].as_str().unwrap(),
                );
                let sender = sender_thread.lock().unwrap();
                println!("sending {}", message.date.to_string());
                sender.send(message);
                println!("sent");
                /*
                let messages = messages.to_owned().write().borrow_mut().into_inner();
                if messages.contains_key(&message.room) {
                    messages.get_mut(&message.room).unwrap().push(message);
                } else {
                    messages.insert(message.room, vec![message]);
                }
                */
            });
        }
        NOT_INITIALIZED = false;
    }

    render! {
        link { rel: "stylesheet", href: "../dist/reset.css" }
        link { rel: "stylesheet", href: "../dist/style.css" }
        Router::<Route> {}
    }
}

fn SideBar(cx: Scope) -> Element {
    render! {
        div {
            id: "sidebar",
            div {
                id: "status",
                "connected"
            }
            div {
                id: "friends",
                Link {
                    class: "friend active",
                    to: Route::Conv{ room: 0 },
                    "Polo"
                }
            }
            form {
                id: "new-friend",
                onsubmit: move |event| {
                    let name = event.data.values.get("name").unwrap().first().unwrap();
                    println!("Submitted! {name:?}")
                },
                input {
                    r#type: "text",
                    name: "name",
                    id: "name",
                    autocomplete: "off",
                    placeholder: "new friend",
                    maxlength: "29"
                }
                input {
                    r#type: "submit",
                    "+"
                }
            }
        }
    }
}

#[inline_props]
fn Conv(cx: Scope, room: usize) -> Element {
    let messages = use_shared_state::<Messages>(cx).unwrap();

    let username = use_state(cx, || String::new());
    let message = use_state(cx, || String::new());

    let send = move |_| {
        let mut username: &str = username;

        if message.is_empty() {
            println!("Empty message");
            return;
        }
        if username.is_empty() {
            username = "guest";
        }

        let form = serialize(room.clone(), 0, message.to_string());

        let url = format!("{BASE_API_URL}/message");
        Runtime::new().unwrap().block_on(async {
            println!("Submitting... {username:?}: {message:?}");
            let _ = reqwest::Client::new().post(&url).form(&form).send().await;
            message.set(String::new());
            println!("Submitted! {username:?}: {message:?}");
        });
        return ();
    };

    let m = messages.read();
    let m2 = m.lock().unwrap();
    let empty = Vec::<Message>::new();
    let vec = m2.get(&room);
    let messages = vec.unwrap_or_else(|| &empty);

    if vec.is_none() {
        println!("display room {} with null elements", room);
    } else {
        println!("display room {} with {} elements", room, messages.len());
    }

    render! {
        SideBar {}
        div{
            id:"content",
            span { room.to_string() }
            span { message.len().to_string() }
            div{
                id: "messages",
                for msg in messages {
                    message_element {
                        date: msg.date,
                        room: msg.room,
                        user_id: msg.user_id,
                        text: msg.text.to_string()
                    }
                }
            }

            form {
                id: "new-message",
                prevent_default: "onsubmit",
                onsubmit: send,
                input {
                    r#type: "text",
                    name: "username",
                    id: "username",
                    autocomplete: "off",
                    placeholder: "guest",
                    maxlength: "19",
                    oninput: move |evt| username.set(evt.value.clone()),
                    value: "{username}"
                }
                input {
                    r#type: "text",
                    name: "message",
                    id: "message",
                    autocomplete: "off",
                    placeholder: "Send a message...",
                    autofocus: true,
                    oninput: move |evt| message.set(evt.value.clone()),
                    value: "{message}"
                }
                button {
                    id: "send",
                    r#type: "submit",
                    "Send"
                }
            }
        }
    }
}

#[inline_props]
fn Home(cx: Scope) -> Element {
    render! {
        SideBar {}
        div{
            id:"content",
            "Home"
        }
    }
}

fn message_element(cx: Scope<Message>) -> Element {
    return render! {
        div{
            class: "message",
            span{
                class: "username",
                cx.props.user_id.to_string()
            },
            span{
                class: "text",
                cx.props.text.as_str()
            }
        }
    };
}

fn main() {
    // launch the web app
    dioxus_desktop::launch(page);
}
