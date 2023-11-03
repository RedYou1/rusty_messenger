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
async fn test_login() {
    initialize();

    let client = Client::tracked(build(true)).await.unwrap();

    let user1 = FormAddUser {
        username: "test1".to_string(),
        password: "test1".to_string(),
    };
    assert_eq!(
        login(&client, &user1)
            .await
            .expect_err("The user test1 shouldn't exists"),
        "no user with the username test1"
    );
    let past_user = add_user(&client, &user1)
        .await
        .expect("The user test1 shouldn't exists");
    let failed_user = FormAddUser {
        username: "test1".to_string(),
        password: "testX".to_string(),
    };
    assert_eq!(
        login(&client, &failed_user)
            .await
            .expect_err("Wrong password"),
        "bad username or password"
    );
    let mut user = login(&client, &user1)
        .await
        .expect("The user test1 should exists");
    assert_eq!(past_user.id, user.id);
    assert_ne!(past_user.api_key, user.api_key);
    assert_eq!(
        user.username,
        get_user(&client, user.id)
            .await
            .expect("The user test1 should exists")
    );

    let room = user
        .addroom(&client, String::from("Room #1"))
        .await
        .expect("can't create Room #1");

    match listen_events(
        &client,
        &UserPass {
            id: 100,
            username: "test".to_string(),
            pass: "test".to_string(),
            api_key: "no_key".to_string(),
        },
    )
    .await
    {
        Ok(_) => {
            panic!("wrong event stream cant succed")
        }
        Err(e) => {
            assert_eq!(e, "bad user id or api key");
        }
    };

    let mut user_events = listen_events(&client, &user).await.unwrap();
    user_events
        .test_next(EventMessage::Room(room.clone()))
        .await;

    let message = user
        .addmessage(&client, room.id, String::from("Salut"))
        .await
        .expect("can't send message");
    user_events
        .test_next(EventMessage::Message(message.clone()))
        .await;

    let mut user2 = add_user(
        &client,
        &FormAddUser {
            username: "test2".to_string(),
            password: "test2".to_string(),
        },
    )
    .await
    .expect("The user test2 should exists");
    assert_eq!(
        user2.username,
        get_user(&client, user2.id)
            .await
            .expect("The user test2 should exists")
    );

    user.invite(&client, user2.username.to_string(), room.id)
        .await
        .expect("can't invite");

    let mut user2_events = listen_events(&client, &user2).await.unwrap();
    user2_events
        .test_next(EventMessage::Room(room.clone()))
        .await;
    user2_events.test_next(EventMessage::Message(message)).await;

    let message2 = user2
        .addmessage(&client, room.id, String::from("Bonjour"))
        .await
        .expect("can't send message");
    user_events
        .test_next(EventMessage::Message(message2.clone()))
        .await;
    user2_events
        .test_next(EventMessage::Message(message2))
        .await;
}

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        dotenv().expect("not .env");
        let database_url = env::var("DATABASE_URL_TEST").expect("DATABASE_URL_TEST must be set");
        if fs::metadata(database_url.clone()).is_ok() {
            fs::remove_file(database_url).expect("can't remove bd");
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
