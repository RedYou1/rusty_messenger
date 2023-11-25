use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::{
    event_source::SourceState, room::OpRoomId, AccountManager, Rooms, Route, BASE_API_URL,
};

#[inline_props]
pub fn SideBar(cx: Scope, room_id: OpRoomId) -> Element {
    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let name = use_state(cx, || String::new());
    let error = use_state::<Option<String>>(cx, || None);

    let state = match *source_state.read() {
        SourceState::Error => "error",
        SourceState::ReConnecting => "reconnecting",
        SourceState::Connected => "connected",
    };

    let rooms = rooms.read();
    let rooms = rooms.0.iter();

    const CLASS_ROOM: &'static str = "room";
    const CLASS_ROOM_ACTIVE: &'static str = "room active";

    render! {
        div {
            id: "sidebar",
            class: match cx.props.room_id.as_ref(){
                Some(Some(_)) => "optional",
                _ => "",
            },
            div {
                id: "status",
                class: state
            }
            ul {
                id: "rooms",
                for (room_id, room_data) in rooms {
                    li {
                        Link {
                            class: match cx.props.room_id.as_ref() {
                                Some(Some(current_room_id)) => if *current_room_id == *room_id { CLASS_ROOM_ACTIVE } else { CLASS_ROOM },
                                _ => CLASS_ROOM
                            },
                            to: Route::Conv{ room_id: *room_id },
                            room_data.name.as_str()
                        }
                    }
                }
            }
            match error.as_ref() {
                Some(e) => render!{span{class:"Error",e.as_str()}},
                None => render!{span{}}
            }
            form {
                id: "new-room",
                input {
                    r#type: "text",
                    name: "name",
                    id: "name",
                    autocomplete: "off",
                    placeholder: "new room",
                    maxlength: "29",
                    oninput: move |evt| name.set(evt.value.clone()),
                    value: "{name}"
                }
                button {
                    id: "send",
                    prevent_default: "onclick",
                    onclick: move |_| create_room(cx, account_manager.to_owned(), name.to_owned(), error.to_owned()),
                    "+"
                }
            }
        }
    }
}

fn create_room<T>(
    cx: Scope<T>,
    account_manager: UseSharedState<AccountManager>,
    name: UseState<String>,
    error: UseState<Option<String>>,
) {
    if name.is_empty() {
        error.set(Some(String::from("Empty room name")));
        return;
    }
    let form: HashMap<&str, String> = {
        let account_manager = account_manager.read();
        let current_user = account_manager.current_user().unwrap();
        HashMap::<&'static str, String>::from([
            ("user_id", current_user.id.to_string()),
            ("api_key", current_user.api_key.to_string()),
            ("name", name.to_string()),
        ])
    };

    let url = format!("{BASE_API_URL}/room");
    cx.spawn(async move {
        match reqwest::Client::new().post(&url).form(&form).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap();
                let response_data = json::parse(response_body.as_str()).unwrap();
                match status {
                    201 => {
                        account_manager
                            .write_silent()
                            .set_api_key(response_data["api_key"].as_str().unwrap().to_string());
                        error.set(None);
                        name.set(String::new());
                    }
                    _ => error.set(Some(response_data["reason"].as_str().unwrap().to_string())),
                }
            }
            Err(_) => error.set(Some(String::from("Request Timeout"))),
        }
    });
}
