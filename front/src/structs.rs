#![allow(non_snake_case)]

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}

pub fn serialize_login(username: String, password: String) -> HashMap<&'static str, String> {
    HashMap::<&'static str, String>::from([("username", username), ("password", password)])
}

pub fn serialize_message(
    room: i64,
    user_id: i64,
    api_key: String,
    text: String,
) -> HashMap<&'static str, String> {
    HashMap::<&'static str, String>::from([
        ("room_id", room.to_string()),
        ("user_id", user_id.to_string()),
        ("api_key", api_key),
        ("text", text),
    ])
}
