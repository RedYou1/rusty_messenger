#![allow(non_snake_case)]

use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use dioxus::prelude::*;

pub struct User {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Clone, PartialEq, Props)]
pub struct Message {
    pub date: DateTime<Utc>,
    pub room: usize,
    pub user_id: usize,
    pub text: String,
}

pub fn deserialize(date: i64, room: usize, user_id: usize, text: &str) -> Message {
    Message {
        date: NaiveDateTime::from_timestamp_opt(date, 0)
            .map(|odt| odt.and_utc())
            .unwrap(),
        room: room,
        user_id: user_id,
        text: text.to_string(),
    }
}

pub fn serialize_login(username: String, password: String) -> HashMap<&'static str, String> {
    return HashMap::<&'static str, String>::from([("username", username), ("password", password)]);
}

pub fn serialize_message(
    room: usize,
    user_id: i64,
    api_key: String,
    text: String,
) -> HashMap<&'static str, String> {
    return HashMap::<&'static str, String>::from([
        ("room", room.to_string()),
        ("user_id", user_id.to_string()),
        ("api_key", api_key),
        ("text", text),
    ]);
}
