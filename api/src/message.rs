use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row};

use crate::db::DateTimeSql;

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

pub fn add_message<'a, 'b>(conn: &'a Connection, message: FormMessage) -> Result<Message> {
    let date = Utc::now();
    conn.execute(
        "INSERT INTO message (date, room, user_id, text) VALUES (?1, ?2, ?3, ?4)",
        (
            date.timestamp(),
            message.room,
            message.user_id,
            message.text.to_string(),
        ),
    )?;

    return Ok(Message {
        date: date,
        room: message.room,
        user_id: message.user_id,
        text: message.text,
    });
}

pub fn load_messages(conn: &Connection, user_id: i64) -> Result<Vec<Message>> {
    let mut stmt =
        conn.prepare("SELECT date, room, user_id, text FROM message WHERE user_id = ?1")?;
    let rows = stmt.query_map([user_id], map_message)?;

    let mut messages = Vec::new();
    for message in rows {
        messages.push(message?);
    }

    return Ok(messages);
}

fn map_message(row: &Row) -> Result<Message> {
    return Ok(Message {
        date: DateTimeSql::parse(row.get(0)?).unwrap(),
        room: row.get(1)?,
        user_id: row.get(2)?,
        text: row.get(3)?,
    });
}
