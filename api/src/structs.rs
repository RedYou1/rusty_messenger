use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct User {
    pub id: usize,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub date: DateTime<Utc>,
    pub room: usize,
    pub user_id: usize,
    pub text: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MessageSerialized {
    pub date: i64,
    pub room: usize,
    pub user_id: usize,
    pub text: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormMessage {
    pub room: usize,
    pub user_id: usize,
    pub text: String,
}

impl Message {
    pub fn serialize(&self) -> MessageSerialized {
        MessageSerialized {
            date: self.date.timestamp(),
            room: self.room,
            user_id: self.user_id,
            text: self.text.clone(),
        }
    }
}
