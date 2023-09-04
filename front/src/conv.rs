use dioxus::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

use crate::side_bar::SideBar;
use crate::structs::{serialize_message, Message, User};
use crate::Messages;
use crate::BASE_API_URL;

#[inline_props]
pub fn Conv(cx: Scope, room: usize) -> Element {
    let user = use_shared_state::<Option<User>>(cx).unwrap();
    let messages = use_shared_state::<Messages>(cx).unwrap();

    let message = use_state(cx, || String::new());

    let send = move |_| {
        if message.is_empty() {
            println!("Empty message");
            return;
        }
        let id: i64;
        let username: String;
        let form: HashMap<&str, String>;
        {
            let r = user.read();
            let user = r.as_ref().unwrap();
            id = user.id;
            username = user.username.to_string();
            form = serialize_message(
                room.clone(),
                user.id,
                user.api_key.to_string(),
                message.to_string(),
            );
        }

        let url = format!("{BASE_API_URL}/message");
        Runtime::new().unwrap().block_on(async {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    let mut u = user.write();
                    *u = Some(User {
                        id: id,
                        username: username,
                        api_key: value["api_key"].as_str().unwrap().to_string(),
                    });
                }
            }
            message.set(String::new());
        });
        return ();
    };

    let m = messages.read();
    let m2 = m.lock().unwrap();
    let messages = m2.get(&room);

    render! {
        SideBar{}
        div{
            id:"content",
            span { room.to_string() }
            span { message.len().to_string() }
            div{
                id: "messages",
                match messages {
                    Some(messages) => render!{
                        for msg in messages {
                            message_element {
                                date: msg.date,
                                room: msg.room,
                                user_id: msg.user_id,
                                text: msg.text.to_string()
                            }
                        }
                    },
                    None => render!{div{}}
                }
            }

            form {
                id: "new-message",
                prevent_default: "onsubmit",
                onsubmit: send,
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
