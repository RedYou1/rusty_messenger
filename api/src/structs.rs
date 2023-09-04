use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Debug)]
pub struct UserPass {
    pub id: i64,
    pub username: String,
    pub pass: String,
    pub api_key: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthKey {
    pub user_id: i64,
    pub api_key: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthPass {
    pub user_id: i64,
    pub password: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormAddUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub date: DateTime<Utc>,
    pub room: usize,
    pub user_id: i64,
    pub text: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MessageSerialized {
    pub date: i64,
    pub room: usize,
    pub user_id: i64,
    pub text: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormMessage {
    pub user_id: i64,
    pub api_key: String,
    pub room: usize,
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
