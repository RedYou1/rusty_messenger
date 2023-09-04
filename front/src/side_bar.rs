use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::{event_source::SourceState, Route};

pub fn SideBar(cx: Scope) -> Element {
    let source_state = use_shared_state::<SourceState>(cx).unwrap();

    let state = match *source_state.read() {
        SourceState::Disconnected => "disconnected",
        SourceState::ReConnecting => "reconnecting",
        SourceState::Connected => "connected",
    };

    render! {
        div {
            id: "sidebar",
            div {
                id: "status",
                class: state
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
