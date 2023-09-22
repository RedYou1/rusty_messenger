use chrono::{DateTime, Utc};
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row};

use crate::db::DateTimeSql;

#[derive(Debug, Clone)]
pub struct Message {
    pub date: DateTime<Utc>,
    pub room_id: i64,
    pub user_id: i64,
    pub text: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormMessage {
    pub user_id: i64,
    pub api_key: String,
    pub room_id: i64,
    pub text: String,
}

impl Message {
    pub fn serialize(&self) -> String {
        format!(
            "{{ \"objectId\": {}, \"date\": {}, \"room_id\": {}, \"user_id\": {}, \"text\": \"{}\" }}",
            0,
            self.date.timestamp(),
            self.room_id,
            self.user_id,
            self.text.clone(),
        )
    }
}

pub fn add_message<'a, 'b>(conn: &'a Connection, message: FormMessage) -> Result<Message> {
    let date = Utc::now();
    conn.execute(
        "INSERT INTO message (date, room_id, user_id, text) VALUES (?1, ?2, ?3, ?4)",
        (
            date.timestamp(),
            message.room_id,
            message.user_id,
            message.text.to_string(),
        ),
    )?;

    Ok(Message {
        date: date,
        room_id: message.room_id,
        user_id: message.user_id,
        text: message.text,
    })
}

pub fn load_messages(conn: &Connection, user_id: i64) -> Result<Vec<Message>> {
    let mut stmt =
        conn.prepare("SELECT message.date, message.room_id, message.user_id, message.text FROM user_room INNER JOIN message ON message.room_id = user_room.room_id WHERE user_room.user_id = ?1 ORDER BY message.date")?;
    let rows = stmt.query_map([user_id], map_message)?;

    let mut messages = Vec::new();
    for message in rows {
        messages.push(message?);
    }

    Ok(messages)
}

fn map_message(row: &Row) -> Result<Message> {
    Ok(Message {
        date: DateTimeSql::parse(row.get(0)?).unwrap(),
        room_id: row.get(1)?,
        user_id: row.get(2)?,
        text: row.get(3)?,
    })
}
