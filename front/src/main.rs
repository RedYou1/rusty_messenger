#![allow(non_snake_case)]

const BASE_API_URL: &str = "http://127.0.0.1:8000";

use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use tokio::runtime::Runtime;

#[derive(Routable, Clone)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/:id")]
    Conv { id: String }
}

fn page(cx: Scope) -> Element {
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
                class: "connected"
            }
            div {
                id: "friends",
                Link {
                    class: "friend",
                    to: Route::Conv{ id: String::from("Polo") },
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
fn Conv(cx: Scope, id: String) -> Element {
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

        let form = HashMap::from([
            ("room", id.as_str()),
            ("username", username),
            ("message", message),
        ]);

        let url = format!("{BASE_API_URL}/message");
        Runtime::new().unwrap().block_on(async {
            println!("Submitting... {username:?}: {message:?}");
            let _ = reqwest::Client::new().post(&url).form(&form).send().await;
            message.set(String::new());
            println!("Submitted! {username:?}: {message:?}");
        });
        return ();
    };

    render! {
        SideBar {}
        div{
            id:"content",
            span { id.as_str() }
            div{
                id: "messages",
                message_element {
                    username: String::from("Polo"),
                    text: String::from("Jack a dit")
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

#[derive(PartialEq, Props)]
struct Message {
    username: String,
    text: String,
}

fn message_element(cx: Scope<Message>) -> Element {
    return render! {
        div{
            id: "messages",
            div{
                class: "message",
                span{
                    class: "username",
                    cx.props.username.as_str()
                },
                span{
                    class: "text",
                    cx.props.text.as_str()
                }
            }
        }
    };
}

fn main() {
    // launch the web app
    dioxus_desktop::launch(page);
}
