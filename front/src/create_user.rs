use dioxus::prelude::Scope;
use dioxus::prelude::*;
use dioxus_router::prelude::Link;

use crate::side_bar::SideBar;
use crate::structs::{serialize_login, User};
use crate::BASE_API_URL;
use crate::{AccountManager, Route};

#[inline_props]
pub fn CreateUser(cx: Scope) -> Element {
    let user = use_shared_state::<AccountManager>(cx).unwrap();
    let username = use_state(cx, || String::new());
    let password = use_state(cx, || String::new());

    let send = move |event: Event<FormData>| {
        event.stop_propagation();

        if username.is_empty() {
            println!("Empty username");
            return;
        }
        if password.is_empty() {
            println!("Empty password");
            return;
        }

        let form = serialize_login(username.to_string(), password.to_string());

        let user = user.to_owned();
        let username = username.to_owned();
        let url = format!("{BASE_API_URL}/adduser");
        cx.spawn(async move {
            let res = reqwest::Client::new().post(&url).form(&form).send().await;
            if res.is_ok() {
                let r = res.unwrap().text().await.unwrap();
                let value = json::parse(r.as_str()).unwrap();
                if value["status_code"].as_u16().unwrap() == 201 {
                    let u = user.write();
                    let mut u = u.lock().unwrap();
                    *u = Some(User {
                        id: value["user_id"].as_i64().unwrap(),
                        username: username.to_string(),
                        api_key: value["api_key"].as_str().unwrap().to_string(),
                    });
                }
            }
        });
    };

    render! {
        match user.read().lock().unwrap().as_ref() {
            Some(_) => render!{SideBar{}},
            None => render!{div{}}
        }
        div{
            id:"createuser",
            h1{"Create User"}
            form {
                id: "create-user",
                prevent_default: "onsubmit",
                onsubmit: send,
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
                    r#type: "submit",
                    "Send"
                }
            }
            Link{
                to: Route::LogIn{},
                "Login"
            }
        }
    }
}
