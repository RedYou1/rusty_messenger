use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::{account::AccountManager, event_source::SourceState, Route};

pub fn SideBar(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap().to_owned();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();

    let state = match *source_state.read() {
        SourceState::Disconnected => "disconnected",
        SourceState::ReConnecting => "reconnecting",
        SourceState::Connected => "connected",
    };

    let user_rooms: &UseFuture<Result<Vec<i64>, String>> = use_future(cx, (), |_| async move {
        //if let Some(a) = user.write_silent().as_mut() {
        //    return a.load_rooms().await;
        //}
        return Err(String::from("not logged in"));
    })
    .to_owned();

    render! {
        div {
            id: "sidebar",
            div {
                id: "status",
                class: state
            }
            div {
                id: "friends",
                match user_rooms.value() {
                    Some(Ok(rooms)) => render!{for room in rooms {
                        Link {
                            class: "friend active",
                            to: Route::Conv{ room: *room },
                            room.to_string()
                        }
                    }},
                    Some(Err(e)) => render!{span{e.to_string()}},
                    None => render!{span{"Loading..."}}
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
