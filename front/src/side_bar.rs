use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::{
    event_source::SourceState,
    room::{OpRoomId, Room},
    AccountManager, Rooms, Route, BASE_API_URL,
};

pub fn SideBar(cx: Scope<OpRoomId>) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let name = use_state(cx, || String::new());

    let state = match *source_state.read() {
        SourceState::Disconnected => "disconnected",
        SourceState::ReConnecting => "reconnecting",
        SourceState::Connected => "connected",
    };

    let send = move |event: Event<FormData>| {
        event.stop_propagation();

        if name.is_empty() {
            println!("Empty message");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let r = user.read();
            let t = r.lock().unwrap();
            let t = t.as_ref().unwrap();
            form = HashMap::<&'static str, String>::from([
                ("user_id", t.id.to_string()),
                ("api_key", t.api_key.to_string()),
                ("name", name.to_string()),
            ]);
        }

        let user = user.to_owned();
        let name = name.to_owned();
        let url = format!("{BASE_API_URL}/room");
        cx.spawn(async move {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    let u = user.write();
                    let mut u = u.lock().unwrap();
                    let l = u.as_mut().unwrap();
                    l.api_key = value["api_key"].as_str().unwrap().to_string();

                    name.set(String::new());
                }
            }
        });
    };

    let m = rooms.read();
    let rooms = m.lock().unwrap();
    let rooms: &Vec<Room> = rooms.as_ref();

    let class_room = "room";
    let class_room_active = "room active";

    render! {
        div {
            id: "sidebar",
            div {
                id: "status",
                class: state
            }
            ul {
                id: "rooms",
                for room in rooms {
                    li {
                        Link {
                            class: match cx.props.id {
                                Some(room_id) => if room.id == room_id { class_room_active } else { class_room },
                                _ => class_room
                            },
                            to: Route::Conv{ room_id: room.id, room_name: room.name.to_string() },
                            room.name.as_str()
                        }
                    }
                }
            }
            form {
                id: "new-room",
                prevent_default: "onsubmit",
                onsubmit: send,
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
                input {
                    r#type: "submit",
                    "+"
                }
            }
        }
    }
}
