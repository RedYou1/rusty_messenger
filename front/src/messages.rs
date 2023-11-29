//! Module de conversation des utilisateur.
//!
//! Ce module implémente la page pour que les utilisateurs puissent ce communiquez dans des salons.

use chrono::Local;
use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use dioxus_router::prelude::Link;
use lib::Message;
use std::collections::HashMap;

use crate::async_state::AsyncStateSetter;
use crate::room::OpRoomId;
use crate::side_bar::SideBar;
use crate::Rooms;
use crate::Route;
use crate::BASE_API_URL;
use crate::{AccountManager, Users};
use lib::serialize_message;

#[inline_props]
pub fn Conv(cx: Scope, room_id: i64) -> Element {
    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();

    let navigator = use_navigator(cx);
    if account_manager.read().utilisateur_actuelle().is_none() {
        navigator.replace(Route::LogIn {});
        return render! {div{}};
    }

    let rooms = use_shared_state::<Rooms>(cx).unwrap();

    let room_data = rooms.read();
    let room_data = room_data.0.get(room_id).unwrap();

    let username = use_state(cx, || String::new());
    let message = use_state(cx, || String::new());
    let error_invite = use_state::<Option<String>>(cx, || None);
    let error_message = use_state::<Option<String>>(cx, || None);

    render! {
        SideBar{room_id: OpRoomId::from(*room_id) }
        div{
            id:"conv",
            div{
                id: "convHeader",
                Link{
                    to: Route::SideBar { room_id: OpRoomId::new_empty() },
                    "<"
                }

                span { room_data.name.as_str() }
            }
            match error_invite.as_ref() {
                Some(e) => render!{span{class:"Error",e.as_str()}},
                None => render!{span{}}
            }
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
                    onclick: move |_| send_invite(cx, account_manager.to_owned(), username.to_owned(), room_id, error_invite.to_owned()),
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
            match error_message.as_ref() {
                Some(e) => render!{span{class:"Error",e.as_str()}},
                None => render!{span{}}
            }
            form {
                id: "new-message",
                input {
                    r#type: "text",
                    name: "message",
                    id: "message",
                    autofocus: true,
                    autocomplete: "off",
                    placeholder: "Send a message...",
                    autofocus: true,
                    oninput: move |evt| message.set(evt.value.clone()),
                    value: "{message}"
                }
                button {
                    id: "send",
                    prevent_default: "onclick",
                    onclick: move |_| send_message(cx, account_manager.to_owned(), message.to_owned(), room_id, error_message.to_owned()),
                    "Send"
                }
            }
        }
    }
}

fn send_message<T>(
    cx: Scope<T>,
    account_manager: UseSharedState<AccountManager>,
    message: UseState<String>,
    room_id: &i64,
    error_message: UseState<Option<String>>,
) {
    if message.is_empty() {
        error_message.set(Some(String::from("Il faut au moins une lettre dans le message")));
        return;
    }
    let form: HashMap<&str, String> = {
        let lock = account_manager.read();
        let current_user = lock.utilisateur_actuelle().unwrap();
        serialize_message(
            *room_id,
            current_user.id,
            current_user.api_key.to_string(),
            message.to_string(),
        )
    };

    let url = format!("{BASE_API_URL}/message");
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
                            .modifier_api_key(response_data["api_key"].as_str().unwrap().to_string());
                        error_message.set(None);
                        message.set(String::new());
                    }
                    _ => error_message.set(Some(response_data["reason"].as_str().unwrap().to_string())),
                }
            }
            Err(_) => error_message.set(Some(String::from("Perte de connection"))),
        }
    });
}

fn send_invite<T>(
    cx: Scope<T>,
    account_manager: UseSharedState<AccountManager>,
    username: UseState<String>,
    room_id: &i64,
    error_invite: UseState<Option<String>>,
) {
    if username.is_empty() {
        error_invite.set(Some(String::from("Il faut au moins une lettre dans le nom")));
        return;
    }
    let form: HashMap<&str, String> = {
        let lock = account_manager.read();
        let current_user = lock.utilisateur_actuelle().unwrap();
        HashMap::<&'static str, String>::from([
            ("user_id", current_user.id.to_string()),
            ("api_key", current_user.api_key.to_string()),
            ("other_user_username", username.to_string()),
            ("room_id", room_id.to_string()),
        ])
    };

    let url = format!("{BASE_API_URL}/invite");
    cx.spawn(async move {
        match reqwest::Client::new().post(&url).form(&form).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap();
                let response_data = json::parse(response_body.as_str()).unwrap();
                match response_data["api_key"].as_str() {
                    Some(api_key) => account_manager
                        .write_silent()
                        .modifier_api_key(api_key.to_string()),
                    None => {}
                }
                match status {
                    201 => {
                        error_invite.set(None);
                        username.set(String::new());
                    }
                    _ => error_invite
                        .set(Some(response_data["reason"].as_str().unwrap().to_string())),
                }
            }
            Err(_) => error_invite.set(Some(String::from("Perte de connection"))),
        }
    });
}

fn message_element<'a, T>(cx: Scope<'a, T>, message: &Message) -> Element<'a> {
    let users = use_shared_state::<Users>(cx).unwrap();
    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();

    let message_user_id = message.user_id;
    let users_setter = AsyncStateSetter::<String>::new(cx, users, move |users, username| {
        users.write().0.insert(message_user_id, Some(username));
    });

    let username = users
        .read()
        .0
        .get(&message_user_id)
        .map(|username| username.as_ref().map(|username| username.to_string()));
    let username = match username {
        Some(Some(username)) => username,
        Some(None) => String::from("Chargement"),
        None => {
            users.write().0.insert(message_user_id, None);
            cx.spawn(async move {
                if let Ok(response) = reqwest::Client::new()
                    .get(format!("{BASE_API_URL}/user/{}", message_user_id))
                    .send()
                    .await
                {
                    let status = response.status().as_u16();
                    let response_body = response.text().await.unwrap();
                    let response_data = json::parse(response_body.as_str()).unwrap();
                    if status == 200 {
                        users_setter
                            .set_state(response_data["username"].as_str().unwrap().to_string());
                    }
                }
            });
            String::from("Chargement")
        }
    };

    const MESSAGE_ME: &'static str = "messageMe";
    const MESSAGE_OTHER: &'static str = "messageOther";

    render! {
        div{
            class: match account_manager.read().utilisateur_actuelle() {
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
