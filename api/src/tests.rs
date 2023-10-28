use dotenv::dotenv;
use json::JsonValue;
use rocket::http::uri::fmt::{Query, UriDisplay};
use rocket::http::ContentType;
use rocket::local::asynchronous::{Client, LocalResponse};
use std::sync::Once;
use std::{env, fs};

use crate::user::UserPass;

use super::*;

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
    let user = login(&client, &user1)
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
    let result = into_json(
        client
            .post(uri!(post_adduser))
            .header(ContentType::Form)
            .body((login as &dyn UriDisplay<Query>).to_string())
            .dispatch()
            .await,
    )
    .await;
    match result["status_code"].as_u16().unwrap() {
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
    let result = into_json(client.get(format!("/user/{}", id)).dispatch().await).await;
    match result["status_code"].as_u16().unwrap() {
        200 => Ok(result["username"].as_str().unwrap().to_string()),
        _ => Err(result["reason"].as_str().unwrap().to_string()),
    }
}

async fn login<'c>(client: &'c Client, login: &FormAddUser) -> Result<UserPass, String> {
    let result = into_json(
        client
            .post(uri!(post_login))
            .header(ContentType::Form)
            .body((login as &dyn UriDisplay<Query>).to_string())
            .dispatch()
            .await,
    )
    .await;
    match result["status_code"].as_u16().unwrap() {
        202 => Ok(UserPass {
            id: result["user_id"].as_i64().unwrap(),
            username: login.username.clone(),
            pass: login.password.clone(),
            api_key: result["api_key"].as_str().unwrap().to_string(),
        }),
        _ => Err(result["reason"].as_str().unwrap().to_string()),
    }
}

async fn into_json<'c>(res: LocalResponse<'c>) -> JsonValue {
    let res = res.into_string().await.unwrap();
    json::parse(res.as_str()).unwrap()
}
