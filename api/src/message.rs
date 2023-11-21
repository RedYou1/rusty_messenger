use chrono::Utc;
use lib::Message;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Result, Row};

use crate::database::{DateTimeSql, Database};

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct FormMessage {
    pub user_id: i64,
    pub api_key: String,
    pub room_id: i64,
    pub text: String,
}

impl Database {
    pub fn add_message<'a, 'b>(&'a self, form: FormMessage) -> Result<Message> {
        let now = Utc::now();
        self.connection.execute(
            "INSERT INTO message (date, room_id, user_id, text) VALUES (?1, ?2, ?3, ?4)",
            (
                now.timestamp(),
                form.room_id,
                form.user_id,
                form.text.to_string(),
            ),
        )?;

        Ok(Message {
            date: now,
            room_id: form.room_id,
            user_id: form.user_id,
            text: form.text,
        })
    }

    pub fn load_messages(&self, user_id: i64) -> Result<Vec<Message>> {
        let mut stmt =
        self.connection.prepare("SELECT message.date, message.room_id, message.user_id, message.text FROM user_room INNER JOIN message ON message.room_id = user_room.room_id WHERE user_room.user_id = ?1 ORDER BY message.date")?;
        let rows = stmt.query_map([user_id], map_message)?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }

        Ok(messages)
    }
}

fn map_message(row: &Row) -> Result<Message> {
    Ok(Message {
        date: DateTimeSql::parse(row.get(0)?).unwrap(),
        room_id: row.get(1)?,
        user_id: row.get(2)?,
        text: row.get(3)?,
    })
}
