//! Gestion des messages dans une application de chat
//!
//! Ce module implémente des méthodes pour gérer l'ajout de messages dans les salons de discussion,
//! ainsi que la récupération de tous les messages associés à un utilisateur dans une base de données.

use chrono::Utc;
use lib::Message;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Result, Row};

use crate::{database::Database, date_time_sql::DateTimeSql};

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
    /// Ajoute un message dans un salon
    pub fn ajout_message(&self, form: FormMessage) -> Result<Message> {
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

    /// Récupère tous les messages
    pub fn recupere_messages(&self, user_id: i64) -> Result<Vec<Message>> {
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
