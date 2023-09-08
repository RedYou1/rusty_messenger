use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use rocket::serde::ser::StdError;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Result};
use std::error::Error;
use std::fmt::Display;
use std::{env, fmt};

use crate::room::Room;

pub struct DateTimeSql(pub NaiveDateTime);

#[derive(Debug, Clone, Copy)]
pub struct DateTimeSqlError(pub i64);

impl DateTimeSql {
    pub fn new(date: i64) -> Option<DateTimeSql> {
        NaiveDateTime::from_timestamp_opt(date, 0).map(|odt| DateTimeSql(odt))
    }

    pub fn parse(date: i64) -> Option<DateTime<Utc>> {
        NaiveDateTime::from_timestamp_opt(date, 0).map(|odt| odt.and_utc())
    }
}

impl Display for DateTimeSqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DateTimeSqlError({})", self.0)
    }
}

impl StdError for DateTimeSqlError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl FromSql for DateTimeSql {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        i64::column_result(value).and_then(|as_i64| match DateTimeSql::new(as_i64) {
            Some(date) => Ok(date),
            None => Err(FromSqlError::Other(Box::new(DateTimeSqlError(as_i64)))),
        })
    }
}

impl ToSql for DateTimeSql {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        Ok(self.0.timestamp().into())
    }
}

pub fn establish_connection() -> Result<Connection> {
    dotenv().expect("not .env");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = Connection::open(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    conn.execute_batch(
        " 
        CREATE TABLE IF NOT EXISTS user
        (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            api_key TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS user_username
            on user (username);

        CREATE TABLE IF NOT EXISTS room
        (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS message
        (
            date INTEGER NOT NULL,
            room_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            text TEXT NOT NULL,

            UNIQUE(room_id, user_id, date),
            FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE,
            FOREIGN KEY(room_id) REFERENCES room(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS user_room
        (
            user_id INTEGER NOT NULL,
            room_id INTEGER NOT NULL,

            PRIMARY KEY(user_id, room_id),
            FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE,
            FOREIGN KEY(room_id) REFERENCES room(id) ON DELETE CASCADE
        );
        ",
    )?;

    return Ok(conn);
}

pub fn load_rooms(conn: &Connection, user_id: i64) -> Result<Vec<Room>> {
    let mut stmt = conn.prepare("SELECT room.id, room.name FROM user_room INNER JOIN room on room.id = user_room.room_id WHERE user_id = ?1")?;
    let rows = stmt.query([user_id])?;

    let m = rows.mapped(|row| {
        Ok(Room {
            id: row.get::<usize, i64>(0)?,
            name: row.get::<usize, String>(1)?,
        })
    });

    return m.collect();
}
