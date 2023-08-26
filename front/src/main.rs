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
        div {
            link { rel: "stylesheet", href: "../dist/reset.css" }
            link { rel: "stylesheet", href: "../dist/style.css" }
            Router::<Route> {}
        }
    }
}

#[inline_props]
fn Conv(cx: Scope, id: String) -> Element {
    render! {
        ul {
            li {
                span { id.as_str() }
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
        ul {
            li {
                Link {
                    to: Route::Conv { id: String::from("Polo") },
                    "Polo"
                }
            }
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
