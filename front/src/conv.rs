use chrono::Local;
use dioxus::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

use crate::side_bar::SideBar;
use crate::structs::{serialize_message, Message};
use crate::Messages;
use crate::BASE_API_URL;
use crate::{AccountManager, Users};

#[inline_props]
pub fn Conv(cx: Scope, room_id: i64, room_name: String) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let messages = use_shared_state::<Messages>(cx).unwrap();

    let username = use_state(cx, || String::new());
    let message = use_state(cx, || String::new());

    let send_message = move |_| {
        if message.is_empty() {
            println!("Empty message");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let r = user.read();
            let t = r.as_ref().unwrap();
            form = serialize_message(
                room_id.clone(),
                t.id,
                t.api_key.to_string(),
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
                    let l = u.as_mut().unwrap();
                    l.api_key = value["api_key"].as_str().unwrap().to_string();
                    message.set(String::new());
                }
            }
        });
        return ();
    };

    let send_invite = move |_| {
        if username.is_empty() {
            println!("Empty username");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let r = user.read();
            let t = r.as_ref().unwrap();
            form = HashMap::<&'static str, String>::from([
                ("user_id", t.id.to_string()),
                ("api_key", t.api_key.to_string()),
                ("user_other", username.to_string()),
                ("room_id", room_id.to_string()),
            ]);
        }

        let url = format!("{BASE_API_URL}/invite");
        Runtime::new().unwrap().block_on(async {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    let mut u = user.write();
                    let l = u.as_mut().unwrap();
                    l.api_key = value["api_key"].as_str().unwrap().to_string();
                    username.set(String::new());
                }
            }
        });
        return ();
    };

    let m = messages.read();
    let m2 = m.lock().unwrap();
    let messages = m2.get(&room_id);

    render! {
        SideBar{id: *room_id}
        div{
            id:"conv",
            span { room_name.as_str() }
            form {
                id: "invite",
                prevent_default: "onsubmit",
                onsubmit: send_invite,
                input {
                    r#type: "text",
                    name: "username",
                    id: "username",
                    autocomplete: "off",
                    placeholder: "Send an invite...",
                    autofocus: true,
                    oninput: move |evt| username.set(evt.value.clone()),
                    value: "{username}"
                }
                button {
                    id: "send",
                    r#type: "submit",
                    "Send"
                }
            }

            div{
                id: "messages",
                match messages {
                    Some(messages) => render!{
                        for msg in messages {
                            message_element {
                                date: msg.date,
                                room_id: msg.room_id,
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
                onsubmit: send_message,
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

const MESSAGE_ME: &'static str = "messageMe";
const MESSAGE_OTHER: &'static str = "messageOther";

fn get_user(users: &UseSharedState<Users>, user_id: i64) -> Option<String> {
    let rusers = users.read();
    let rusers = rusers.lock().unwrap();
    let r = rusers.get(&user_id);
    r.map(|u| u.to_string())
}

fn message_element(cx: Scope<Message>) -> Element {
    let users = use_shared_state::<Users>(cx).unwrap();
    let user = use_shared_state::<AccountManager>(cx).unwrap();

    let username = match get_user(users, cx.props.user_id) {
        Some(username) => username,
        None => {
            let url = format!("{BASE_API_URL}/user/{}", cx.props.user_id);
            Runtime::new().unwrap().block_on(async {
                let res = reqwest::Client::new().get(&url).send().await;
                if res.is_ok() {
                    let r = res.unwrap().text().await.unwrap();
                    let value = json::parse(r.as_str()).unwrap();
                    if value["status_code"].as_u16().unwrap() == 200 {
                        let u = users.write();
                        let mut u = u.lock().unwrap();
                        u.insert(
                            cx.props.user_id,
                            value["username"].as_str().unwrap().to_string(),
                        );
                    }
                }
            });
            String::from("Loading...")
        }
    };

    return render! {
        div{
            class: match user.read().as_ref() {
                Some(user) => if user.id == cx.props.user_id { MESSAGE_ME } else { MESSAGE_OTHER },
                None => MESSAGE_OTHER
            },
            div{
                class: "message-header",
                span{
                    class: "message-username",
                    username
                }
                span{
                    class: "message-date",
                    cx.props.date.with_timezone(&Local).naive_local().to_string()
                }
            }
            span{
                class: "message-text",
                cx.props.text.as_str()
            }
        }
    };
}
