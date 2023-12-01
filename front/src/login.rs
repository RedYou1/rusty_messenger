//! Module de connection des utilisateur.
//!
//! Ce module implémente la page de connection des utilisateur.

use dioxus::prelude::*;
use dioxus_router::prelude::{use_navigator, Link, Navigator};
use lib::serialize_login;

use crate::async_state::AsyncStateSetter;
use crate::room::OpRoomId;
use crate::structs::User;
use crate::BASE_API_URL;
use crate::{AccountManager, Route};

#[inline_props]
pub fn LogIn(cx: Scope) -> Element {
    let account_manager = use_shared_state::<AccountManager>(cx).unwrap();
    let username = use_state(cx, || String::new());
    let password = use_state(cx, || String::new());
    let error = use_state::<Option<String>>(cx, || None);

    let userSetter =
        AsyncStateSetter::<Option<User>>::new(cx, account_manager, |account_manager, user| {
            account_manager.write().modifier_utilisateur_actuelle(user)
        });

    let navigator = use_navigator(cx);

    render! {
        div{
            id:"login",
            h1{"Connection d'utilisateur"}
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
                    placeholder: "nom d'utilisateur",
                    oninput: move |evt| username.set(evt.value.clone()),
                    value: "{username}"
                }
                input {
                    r#type: "password",
                    name: "password",
                    id: "password",
                    autocomplete: "off",
                    placeholder: "mot de passe",
                    oninput: move |evt| password.set(evt.value.clone()),
                    value: "{password}"
                }
                button {
                    id: "send",
                    prevent_default: "onclick",
                    onclick: move |_| login(cx, navigator.to_owned(), userSetter.to_owned(), username.to_owned(), password, error.to_owned()),
                    "Envoyer"
                }
            }
            Link{
                to: Route::CreateUser{},
                "Création d'utilisateur"
            }
        }
    }
}

/// Envoie une requête de connexion d'utilisateur et se connecte si la requête réussie
fn login<T>(
    cx: Scope<T>,
    navigator: Navigator,
    userSetter: AsyncStateSetter<Option<User>>,
    username: UseState<String>,
    password: &UseState<String>,
    error: UseState<Option<String>>,
) {
    if username.is_empty() {
        error.set(Some(String::from("Il faut au moins une lettre dans le nom")));
        return;
    }
    if password.is_empty() {
        error.set(Some(String::from("Il faut au moins une lettre dans le mot de passe")));
        return;
    }
    let form = serialize_login(username.to_string(), password.to_string());

    let url = format!("{BASE_API_URL}/login");
    cx.spawn(async move {
        match reqwest::Client::new().post(url).form(&form).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_body = response.text().await.unwrap();
                let response_data = json::parse(response_body.as_str()).unwrap();
                match status {
                    202 => {
                        userSetter.set_state(Some(User {
                            id: response_data["user_id"].as_i64().unwrap(),
                            username: username.to_string(),
                            api_key: response_data["api_key"].as_str().unwrap().to_string(),
                        }));
                        navigator.replace(Route::SideBar {
                            room_id: OpRoomId::new_empty(),
                        });
                    }
                    _ => error.set(Some(response_data["reason"].as_str().unwrap().to_string())),
                }
            }
            Err(_) => error.set(Some(String::from("Perte de connection"))),
        }
    });
}
