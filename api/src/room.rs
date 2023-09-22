use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row};

use crate::user::user_select_username;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Room {
    pub id: i64,
    pub name: String,
}

impl Room {
    pub fn serialize(&self) -> String {
        format!(
            "{{ \"objectId\": {}, \"id\": {}, \"name\": \"{}\" }}",
            1,
            self.id,
            self.name.clone(),
        )
    }
}

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

pub fn add_room<'a>(conn: &'a Connection, room: FormAddRoom) -> Result<Room> {
    conn.execute("INSERT INTO room (name) VALUES (?1)", (room.name.as_str(),))?;

    let nroom = Room {
        id: conn.last_insert_rowid(),
        name: room.name,
    };

    conn.execute(
        "INSERT INTO user_room (room_id, user_id) VALUES (?1,?2)",
        (nroom.id, room.user_id),
    )?;

    Ok(nroom)
}

pub fn add_user_room<'a>(
    conn: &'a Connection,
    form: FormAddUserRoom,
) -> Result<(Room, i64), String> {
    let room = room_select_id(conn, form.room_id)?;
    let other = user_select_username(conn, form.user_other.as_str())?;

    match conn.execute(
        "INSERT INTO user_room (user_id, room_id) VALUES (?1, ?2)",
        (other.id, form.user_id),
    ) {
        Ok(_) => Ok((room, other.id)),
        Err(_) => Err(format!("cant prepare")),
    }
}

pub fn room_select_id<'a>(conn: &'a Connection, room_id: i64) -> Result<Room, String> {
    let mut stmt = conn
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

pub fn select_users_room<'a>(conn: &'a Connection, room_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT user_id FROM user_room WHERE room_id = ?1")?;
    let rows = stmt.query([room_id])?;
    let m = rows.mapped(|row| Ok(row.get::<usize, i64>(0)?));
    m.collect()
}

fn map_room(row: &Row) -> Result<Room> {
    Ok(Room {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}
