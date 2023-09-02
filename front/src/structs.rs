#![allow(non_snake_case)]

use std::collections::HashMap;

use chrono::{DateTime, NaiveDateTime, Utc};
use dioxus::prelude::*;

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

pub fn serialize(room: usize, user_id: usize, text: String) -> HashMap<&'static str, String> {
    return HashMap::<&'static str, String>::from([
        ("room", room.to_string()),
        ("user_id", user_id.to_string()),
        ("text", text),
    ]);
}
