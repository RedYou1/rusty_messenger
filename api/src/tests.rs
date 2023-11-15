use chrono::Utc;
use dotenv::dotenv;
use json::JsonValue;
use lib::{EventMessage, Message, Room};
use rocket::http::uri::fmt::{Query, UriDisplay};
use rocket::http::ContentType;
use rocket::local::asynchronous::{Client, LocalResponse};
use std::sync::Once;
use std::{env, fs};

use crate::test_event_stream::listen_events;
use crate::user::UserPass;

use super::*;

static mut ROOMID: i64 = 0;

pub fn next_room_id() -> i64 {
    unsafe {
        ROOMID += 1;
        ROOMID
    }
}

#[async_test]
async fn test_adduser() {
    initialize();
    let client = Client::tracked(build(true)).await.unwrap();
    let form_user = FormAddUser {
        username: "test_adduser".to_string(),
        password: "test_adduser".to_string(),
    };
    let user = add_user(&client, &form_user).await.unwrap();
    assert_eq!(user.username, "test_adduser");
    assert_eq!(user.pass, "test_adduser");
    assert_eq!(
        add_user(&client, &form_user).await.unwrap_err(),
        "Username Already Taken"
    );
}

#[async_test]
async fn test_login() {
    initialize();
    let client = Client::tracked(build(true)).await.unwrap();

    let form_user = FormAddUser {
        username: "test_login".to_string(),
        password: "test_login".to_string(),
    };
    assert_eq!(
        login(&client, &form_user).await.unwrap_err(),
        "bad username or password"
    );
    let add_user = add_user(&client, &form_user).await.unwrap();
    let failed_user = FormAddUser {
        username: "test_login".to_string(),
        password: "Wrong password".to_string(),
    };
    assert_eq!(
        login(&client, &failed_user).await.unwrap_err(),
        "bad username or password"
    );
    let user = login(&client, &form_user).await.unwrap();
    assert_eq!(add_user.id, user.id);
    assert_ne!(add_user.api_key, user.api_key);
    assert_eq!(user.username, get_user(&client, user.id).await.unwrap());
}

#[async_test]
async fn test_room() {
    initialize();
    let client = Client::tracked(build(true)).await.unwrap();

    assert_eq!(
        UserPass {
            id: 100,
            username: "test_room".to_string(),
            pass: "test_room".to_string(),
            api_key: "no key".to_string(),
        }
        .addroom(&client, String::from("Room #1"))
        .await
        .unwrap_err(),
        "bad user id or api key"
    );
}

#[async_test]
async fn test_wrong_event() {
    initialize();
    let client = Client::tracked(build(true)).await.unwrap();

    let user_1 = add_user(
        &client,
        &FormAddUser {
            username: "test_wrong_event".to_string(),
            password: "test_wrong_event".to_string(),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        listen_events(
            &client,
            &UserPass {
                id: user_1.id,
                username: user_1.username.to_string(),
                pass: user_1.pass.to_string(),
                api_key: "wrong_key".to_string(),
            },
        )
        .await
        .unwrap_err(),
        "bad user id or api key"
    );
}

#[async_test]
async fn test_event() {
    initialize();
    let client = Client::tracked(build(true)).await.unwrap();

    let mut user_1 = add_user(
        &client,
        &FormAddUser {
            username: "test_event_1".to_string(),
            password: "test_event_1".to_string(),
        },
    )
    .await
    .unwrap();

    let room = user_1
        .addroom(&client, String::from("Room #1"))
        .await
        .unwrap();

    let mut user_1_events = listen_events(&client, &user_1).await.unwrap();
    user_1_events
        .test_next(EventMessage::Room(room.clone()))
        .await;

    let message = user_1
        .addmessage(&client, room.id, String::from("Salut"))
        .await
        .unwrap();
    user_1_events
        .test_next(EventMessage::Message(message.clone()))
        .await;

    let mut user_2 = add_user(
        &client,
        &FormAddUser {
            username: "test_event_2".to_string(),
            password: "test_event_2".to_string(),
        },
    )
    .await
    .unwrap();
    assert_eq!(user_2.username, get_user(&client, user_2.id).await.unwrap());

    user_1
        .invite(&client, user_2.username.to_string(), room.id)
        .await
        .unwrap();

    let mut user_2_events = listen_events(&client, &user_2).await.unwrap();
    user_2_events
        .test_next(EventMessage::Room(room.clone()))
        .await;
    user_2_events
        .test_next(EventMessage::Message(message))
        .await;

    let message2 = user_2
        .addmessage(&client, room.id, String::from("Bonjour"))
        .await
        .unwrap();
    user_1_events
        .test_next(EventMessage::Message(message2.clone()))
        .await;
    user_2_events
        .test_next(EventMessage::Message(message2))
        .await;
}

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        dotenv().unwrap();
        let database_url = env::var("DATABASE_URL_TEST").unwrap();
        if fs::metadata(database_url.clone()).is_ok() {
            fs::remove_file(database_url).unwrap();
        }
    });
}

async fn add_user<'c>(client: &'c Client, login: &FormAddUser) -> Result<UserPass, String> {
    let response = client
        .post(uri!(post_adduser))
        .header(ContentType::Form)
        .body((login as &dyn UriDisplay<Query>).to_string())
        .dispatch()
        .await;
    let status = response.status().code;
    let result = into_json(response).await;
    match status {
        201 => Ok(UserPass {
            id: result["user_id"].as_i64().unwrap(),
            username: login.username.clone(),
            pass: login.password.clone(),
            api_key: result["api_key"].as_str().unwrap().to_string(),
        }),
        _ => Err(result["reason"].as_str().unwrap().to_string()),
    }
}

async fn get_user<'c>(client: &'c Client, id: i64) -> Result<String, String> {
    let response = client.get(format!("/user/{}", id)).dispatch().await;
    let status = response.status().code;
    let result = into_json(response).await;
    match status {
        200 => Ok(result["username"].as_str().unwrap().to_string()),
        _ => Err(result["reason"].as_str().unwrap().to_string()),
    }
}

async fn login<'c>(client: &'c Client, login: &FormAddUser) -> Result<UserPass, String> {
    let response = client
        .post(uri!(post_login))
        .header(ContentType::Form)
        .body((login as &dyn UriDisplay<Query>).to_string())
        .dispatch()
        .await;
    let status = response.status().code;
    let result = into_json(response).await;
    match status {
        202 => Ok(UserPass {
            id: result["user_id"].as_i64().unwrap(),
            username: login.username.clone(),
            pass: login.password.clone(),
            api_key: result["api_key"].as_str().unwrap().to_string(),
        }),
        _ => Err(result["reason"].as_str().unwrap().to_string()),
    }
}

impl UserPass {
    async fn addroom<'c>(&mut self, client: &'c Client, name: String) -> Result<Room, String> {
        let room = FormAddRoom {
            user_id: self.id,
            api_key: self.api_key.to_string(),
            name: name,
        };
        let response = client
            .post(uri!(post_addroom))
            .header(ContentType::Form)
            .body((&room as &dyn UriDisplay<Query>).to_string())
            .dispatch()
            .await;
        let status = response.status().code;
        let result = into_json(response).await;
        match status {
            201 => {
                self.api_key = result["api_key"].as_str().unwrap().to_string();
                Ok(Room {
                    id: next_room_id(),
                    name: room.name.to_string(),
                })
            }
            _ => Err(result["reason"].as_str().unwrap().to_string()),
        }
    }

    async fn invite<'c>(
        &mut self,
        client: &'c Client,
        other_user: String,
        room: i64,
    ) -> Result<(), String> {
        let room = FormAddUserRoom {
            user_id: self.id,
            api_key: self.api_key.to_string(),
            user_other: other_user,
            room_id: room,
        };
        let response = client
            .post(uri!(post_invite))
            .header(ContentType::Form)
            .body((&room as &dyn UriDisplay<Query>).to_string())
            .dispatch()
            .await;
        let status = response.status().code;
        let result = into_json(response).await;
        match result["api_key"].as_str() {
            Some(api_key) => self.api_key = api_key.to_string(),
            None => {}
        }
        match status {
            201 => Ok(()),
            _ => Err(result["reason"].as_str().unwrap().to_string()),
        }
    }

    async fn addmessage<'c>(
        &mut self,
        client: &'c Client,
        room_id: i64,
        text: String,
    ) -> Result<Message, String> {
        let message = FormMessage {
            user_id: self.id,
            api_key: self.api_key.to_string(),
            room_id: room_id,
            text: text,
        };
        let response = client
            .post(uri!(post_message))
            .header(ContentType::Form)
            .body((&message as &dyn UriDisplay<Query>).to_string())
            .dispatch()
            .await;
        let status = response.status().code;
        let result = into_json(response).await;
        match result["api_key"].as_str() {
            Some(api_key) => self.api_key = api_key.to_string(),
            None => {}
        }
        match status {
            201 => Ok(Message {
                date: Utc::now(),
                room_id: message.room_id,
                user_id: message.user_id,
                text: message.text.to_string(),
            }),
            _ => Err(result["reason"].as_str().unwrap().to_string()),
        }
    }
}

pub async fn into_json<'c>(res: LocalResponse<'c>) -> JsonValue {
    let res = res.into_string().await.unwrap();
    json::parse(res.as_str()).unwrap()
}
