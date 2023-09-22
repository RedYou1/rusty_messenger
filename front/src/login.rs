use dioxus::prelude::*;
use dioxus_router::prelude::Link;

use crate::async_state::AsyncStateSetter;
use crate::side_bar::SideBar;
use crate::structs::{serialize_login, User};
use crate::BASE_API_URL;
use crate::{AccountManager, Route};

#[inline_props]
pub fn LogIn(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let username = use_state(cx, || String::new());
    let password = use_state(cx, || String::new());

    let userSetter = AsyncStateSetter::<Option<User>>::new(cx, user, |account_manager, user| {
        account_manager.write().set_user(user)
    });

    let send = move |_| {
        if username.is_empty() {
            println!("Empty username");
            return;
        }
        if password.is_empty() {
            println!("Empty password");
            return;
        }
        let form = serialize_login(username.to_string(), password.to_string());

        let url = format!("{BASE_API_URL}/login");
        let username = username.to_string();
        let userSetter = userSetter.clone();
        cx.spawn(async move {
            if let Ok(res) = reqwest::Client::new().post(url).form(&form).send().await {
                let r = res.text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 202 {
                    userSetter.set_state(Some(User {
                        id: value["user_id"].as_i64().unwrap(),
                        username: username,
                        api_key: value["api_key"].as_str().unwrap().to_string(),
                    }))
                }
            }
        });
    };

    render! {
        match user.read().user() {
            Some(_) => render!{SideBar{}},
            None => render!{div{}}
        }
        div{
            id:"login",
            h1{"Login"}
            form {
                id: "login-form",
                input {
                    r#type: "text",
                    name: "username",
                    id: "username",
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
