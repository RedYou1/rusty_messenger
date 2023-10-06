use lib::Room;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Result, Row};

use crate::db::MyConnection;

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormAddRoom {
    pub user_id: i64,
    pub api_key: String,
    pub name: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormAddUserRoom {
    pub user_id: i64,
    pub api_key: String,
    pub user_other: String,
    pub room_id: i64,
}

impl MyConnection {
    pub fn add_room<'a>(&'a self, room: FormAddRoom) -> Result<Room> {
        self.conn
            .execute("INSERT INTO room (name) VALUES (?1)", (room.name.as_str(),))?;

        let nroom = Room {
            id: self.conn.last_insert_rowid(),
            name: room.name,
        };

        self.conn.execute(
            "INSERT INTO user_room (room_id, user_id) VALUES (?1,?2)",
            (nroom.id, room.user_id),
        )?;

        Ok(nroom)
    }

    pub fn add_user_room<'a>(&'a self, form: FormAddUserRoom) -> Result<(Room, i64), String> {
        let room = self.room_select_id(form.room_id)?;
        let other = self.user_select_username(form.user_other.as_str())?;

        match self.conn.execute(
            "INSERT INTO user_room (user_id, room_id) VALUES (?1, ?2)",
            (other.id, form.user_id),
        ) {
            Ok(_) => Ok((room, other.id)),
            Err(_) => Err(format!("Can't invite that user.")),
        }
    }

    pub fn room_select_id<'a>(&'a self, room_id: i64) -> Result<Room, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM room WHERE id = ?1")
            .map_err(|_| format!("cant prepare"))?;

        let rows = stmt
            .query_map([room_id], map_room)
            .map_err(|_| format!("cant querry"))?;

        let mut obduser = None;
        for usr in rows {
            if obduser.is_some() {
                return Err(format!("multiple rooms with the id {}", room_id));
            }
            obduser = Some(usr.map_err(|usr| format!("bad room {}", usr.to_string()))?);
        }
        match obduser {
            Some(obduser) => Ok(obduser),
            None => Err(format!("no room with the id {}", room_id)),
        }
    }

    pub fn select_users_room<'a>(&'a self, room_id: i64) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
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
