#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;

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
