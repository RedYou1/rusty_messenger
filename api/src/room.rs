use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row};

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

    return Ok(nroom);
}

pub fn room_select_id<'a>(conn: &'a Connection, room_id: i64) -> Result<Room, String> {
    let stmt = conn.prepare("SELECT id, name FROM room WHERE id = ?1");

    if stmt.is_err() {
        return Err(format!("cant prepare"));
    }

    let mut stmtmut = stmt.unwrap();

    let rows = stmtmut.query_map([room_id], map_room);

    if rows.is_err() {
        return Err(format!("cant querry"));
    }

    let mut obduser = None;
    for usr in rows.unwrap() {
        if obduser.is_some() {
            return Err(format!("multiple rooms with the id {}", room_id));
        }
        if usr.is_err() {
            return Err(format!("bad room {}", usr.unwrap_err().to_string()));
        }
        obduser = Some(usr.unwrap());
    }
    if obduser.is_none() {
        return Err(format!("no room with the id {}", room_id));
    }
    return Ok(obduser.unwrap());
}

fn map_room(row: &Row) -> Result<Room> {
    return Ok(Room {
        id: row.get(0)?,
        name: row.get(1)?,
    });
}
