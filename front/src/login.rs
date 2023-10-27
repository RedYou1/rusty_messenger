use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link};

use crate::async_state::AsyncStateSetter;
use crate::room::OpRoomId;
use crate::structs::{serialize_login, User};
use crate::BASE_API_URL;
use crate::{AccountManager, Route};

#[inline_props]
pub fn LogIn(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let username = use_state(cx, || String::new());
    let password = use_state(cx, || String::new());
    let error = use_state(cx, || None);

    let userSetter = AsyncStateSetter::<Option<User>>::new(cx, user, |account_manager, user| {
        account_manager.write().set_user(user)
    });

    let nav = use_navigator(cx);

    let send = move |_| {
        if username.is_empty() {
            error.set(Some(String::from("Empty username")));
            return;
        }
        if password.is_empty() {
            error.set(Some(String::from("Empty password")));
            return;
        }
        let form = serialize_login(username.to_string(), password.to_string());

        let url = format!("{BASE_API_URL}/login");
        let username = username.to_owned();
        let error = error.to_owned();
        let userSetter = userSetter.clone();
        let nav = nav.to_owned();
        cx.spawn(async move {
            match reqwest::Client::new().post(url).form(&form).send().await {
                Ok(res) => {
                    let r = res.text().await.unwrap();
                    let value = json::parse(r.as_str()).unwrap();
                    match value["status_code"].as_u16().unwrap() {
                        202 => {
                            userSetter.set_state(Some(User {
                                id: value["user_id"].as_i64().unwrap(),
                                username: username.to_string(),
                                api_key: value["api_key"].as_str().unwrap().to_string(),
                            }));
                            nav.replace(Route::SideBar {
                                room_id: OpRoomId::new_empty(),
                            });
                        }
                        _ => error.set(Some(value["reason"].as_str().unwrap().to_string())),
                    }
                }
                Err(_) => error.set(Some(String::from("Request Timeout"))),
            }
        });
    };

    render! {
        div{
            id:"login",
            h1{"Login"}
            match error.as_ref() {
                Some(e) => render!{span{class:"Error",e.as_str()}},
                None => render!{span{}}
            }
            form {
                id: "login-form",
                input {
                    r#type: "text",
                    name: "username",
                    id: "username",
                    autofocus: true,
                    autocomplete: "off",
                    placeholder: "username",
                    oninput: move |evt| username.set(evt.value.clone()),
                    value: "{username}"
                }
                input {
                    r#type: "password",
                    name: "password",
                    id: "password",
                    autocomplete: "off",
                    placeholder: "password",
                    oninput: move |evt| password.set(evt.value.clone()),
                    value: "{password}"
                }
                button {
                    id: "send",
                    prevent_default: "onclick",
                    onclick: send,
                    "Send"
                }
            }
            Link{
                to: Route::CreateUser{},
                "create user"
            }
        }
    }
}
