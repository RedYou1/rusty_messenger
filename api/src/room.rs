use lib::Room;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Result, Row};

use crate::database::Database;

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct FormAddRoom {
    pub user_id: i64,
    pub api_key: String,
    pub name: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct FormAddUserRoom {
    pub user_id: i64,
    pub api_key: String,
    pub other_user_username: String,
    pub room_id: i64,
}

impl Database {
    pub fn add_room<'a>(&'a self, form: FormAddRoom) -> Result<Room> {
        self.connection
            .execute("INSERT INTO room (name) VALUES (?1)", (form.name.as_str(),))?;

        let new_room = Room {
            id: self.connection.last_insert_rowid(),
            name: form.name,
        };

        self.connection.execute(
            "INSERT INTO user_room (room_id, user_id) VALUES (?1,?2)",
            (new_room.id, form.user_id),
        )?;

        Ok(new_room)
    }

    pub fn add_user_room<'a>(&'a self, form: FormAddUserRoom) -> Result<(Room, i64), String> {
        let room = self.room_select_id(form.room_id)?;
        let other_user = self.user_select_username(form.other_user_username.as_str())?;

        match self.connection.execute(
            "INSERT INTO user_room (user_id, room_id) SELECT ?1, ?2 FROM user_room WHERE user_id = ?3 AND room_id = ?2",
            (other_user.id, form.room_id, form.user_id),
        ) {
            Ok(0) => Err(String::from("You can't invite someone in a room you aren't in.")),
            Ok(_) => Ok((room, other_user.id)),
            Err(_) => Err(String::from("That user is already in that room.")),
        }
    }

    pub fn room_select_id<'a>(&'a self, room_id: i64) -> Result<Room, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, name FROM room WHERE id = ?1")
            .map_err(|_| format!("cant prepare"))?;

        let mut rows = stmt
            .query_map([room_id], map_room)
            .map_err(|_| format!("cant querry"))?;

        match rows.next() {
            Some(Ok(bd_room)) => Ok(bd_room),
            _ => Err(format!("no room with the id {}", room_id)),
        }
    }

    pub fn select_users_room<'a>(&'a self, room_id: i64) -> Result<Vec<i64>> {
        let mut stmt = self
            .connection
            .prepare("SELECT user_id FROM user_room WHERE room_id = ?1")?;
        let rows = stmt.query([room_id])?;
        let m = rows.mapped(|row| Ok(row.get::<usize, i64>(0)?));
        m.collect()
    }
}

fn map_room(row: &Row) -> Result<Room> {
    Ok(Room {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}
