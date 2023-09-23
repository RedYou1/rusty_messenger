use chrono::Local;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use lib::Message;
use std::collections::HashMap;

use crate::async_state::AsyncStateSetter;
use crate::side_bar::SideBar;
use crate::structs::serialize_message;
use crate::Rooms;
use crate::Route;
use crate::BASE_API_URL;
use crate::{AccountManager, Users};

#[inline_props]
pub fn Conv(cx: Scope, room_id: i64) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();

    let nav = use_navigator(cx);
    if user.read().user().is_none() {
        nav.replace(Route::LogIn {});
        return render! {div{}};
    }

    let rooms = use_shared_state::<Rooms>(cx).unwrap();

    let room_data = rooms.read();
    let room_data = room_data.0.get(room_id).unwrap();

    let username = use_state(cx, || String::new());
    let message = use_state(cx, || String::new());

    let send_message = move |_| {
        let user_clone = user.to_owned();
        let message_clone = message.to_owned();
        if message_clone.is_empty() {
            println!("Empty message");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let t = user_clone.read();
            let t = t.user().unwrap();
            form = serialize_message(
                *room_id,
                t.id,
                t.api_key.to_string(),
                message_clone.to_string(),
            );
        }

        let url = format!("{BASE_API_URL}/message");
        cx.spawn(async move {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    user_clone
                        .write_silent()
                        .set_api_key(value["api_key"].as_str().unwrap().to_string());
                    message_clone.set(String::new());
                }
            }
        });
    };

    let send_invite = move |_| {
        let user = user.to_owned();
        let username = username.to_owned();
        if username.is_empty() {
            println!("Empty username");
            return;
        }
        let form: HashMap<&str, String>;
        {
            let t = user.read();
            let t = t.user().unwrap();
            form = HashMap::<&'static str, String>::from([
                ("user_id", t.id.to_string()),
                ("api_key", t.api_key.to_string()),
                ("user_other", username.to_string()),
                ("room_id", room_id.to_string()),
            ]);
        }

        let url = format!("{BASE_API_URL}/invite");
        cx.spawn(async move {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    user.write_silent()
                        .set_api_key(value["api_key"].as_str().unwrap().to_string());
                    username.set(String::new());
                }
            }
        });
    };

    render! {
        SideBar{id: *room_id}
        div{
            id:"conv",
            span { room_data.name.as_str() }
            form {
                id: "invite",
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
                    prevent_default: "onclick",
                    onclick: send_invite,
                    "Send"
                }
            }

            div{
                id: "messages",
                match room_data.messages.is_empty() {
                    true => render!{div{}},
                    false => render!{
                        for msg in room_data.messages.iter() {
                            message_element(cx, msg)
                        }
                    },
                }
            }

            form {
                id: "new-message",
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
                    prevent_default: "onclick",
                    onclick: send_message,
                    "Send"
                }
            }
        }
    }
}

const MESSAGE_ME: &'static str = "messageMe";
const MESSAGE_OTHER: &'static str = "messageOther";

fn message_element<'a, T>(cx: Scope<'a, T>, message: &Message) -> Element<'a> {
    let users = use_shared_state::<Users>(cx).unwrap();
    let user = use_shared_state::<AccountManager>(cx).unwrap();

    let message_user_id = message.user_id;
    let users_setter = AsyncStateSetter::<String>::new(cx, users, move |users, username| {
        users.write().0.insert(message_user_id, Some(username));
    });

    let username;
    {
        let t = users.read();
        username =
            t.0.get(&message_user_id)
                .map(|s| s.as_ref().map(|s| s.to_string()))
    }
    let username = match username {
        Some(Some(username)) => username.to_string(),
        Some(None) => String::from("Loading..."),
        None => {
            users.write().0.insert(message_user_id, None);
            cx.spawn(async move {
                let res = reqwest::Client::new()
                    .get(format!("{BASE_API_URL}/user/{}", message_user_id))
                    .send()
                    .await;
                if res.is_ok() {
                    let r = res.unwrap().text().await.unwrap();
                    let value = json::parse(r.as_str()).unwrap();
                    if value["status_code"].as_u16().unwrap() == 200 {
                        users_setter.set_state(value["username"].as_str().unwrap().to_string());
                    }
                }
            });
            String::from("Loading...")
        }
    };

    render! {
        div{
            class: match user.read().user() {
                Some(user) => if user.id == message_user_id { MESSAGE_ME } else { MESSAGE_OTHER },
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
                    message.date.with_timezone(&Local).naive_local().to_string()
                }
            }
            span{
                class: "message-text",
                message.text.as_str()
            }
        }
    }
}
