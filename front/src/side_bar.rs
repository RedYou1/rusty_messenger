use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use tokio::runtime::Runtime;

use crate::{account::AccountManager, event_source::SourceState, Rooms, Route, BASE_API_URL};

pub fn SideBar(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let source_state = use_shared_state::<SourceState>(cx).unwrap();
    let rooms = use_shared_state::<Rooms>(cx).unwrap();
    let name = use_state(cx, || String::new());

    let state = match *source_state.read() {
        SourceState::Disconnected => "disconnected",
        SourceState::ReConnecting => "reconnecting",
        SourceState::Connected => "connected",
    };

    let send = move |_| {
        if name.is_empty() {
            println!("Empty message");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let r = user.read();
            let t = r.as_ref().unwrap();
            form = HashMap::<&'static str, String>::from([
                ("user_id", t.user.id.to_string()),
                ("api_key", t.user.api_key.to_string()),
                ("name", name.to_string()),
            ]);
        }

        let url = format!("{BASE_API_URL}/room");
        Runtime::new().unwrap().block_on(async {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    let mut u = user.write();
                    let l = u.as_mut().unwrap();
                    l.user.api_key = value["api_key"].as_str().unwrap().to_string();

                    name.set(String::new());
                }
            }
        });
        return ();
    };

    let m = rooms.read();
    let m2 = m.lock().unwrap();
    let rooms = m2.as_ref();

    render! {
        div {
            id: "sidebar",
            div {
                id: "status",
                class: state
            }
            div {
                id: "friends",
                for room in rooms {
                    Link {
                        class: "friend active",
                        to: Route::Conv{ room: room.id },
                        room.name.as_str()
                    }
                }
            }
            form {
                id: "new-friend",
                prevent_default: "onsubmit",
                onsubmit: send,
                input {
                    r#type: "text",
                    name: "name",
                    id: "name",
                    autocomplete: "off",
                    placeholder: "new friend",
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
