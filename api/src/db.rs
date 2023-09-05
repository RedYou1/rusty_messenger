use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use pwhash::bcrypt;
use rocket::serde::ser::StdError;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Result, Row};
use std::error::Error;
use std::fmt::Display;
use std::{env, fmt};

use crate::structs::{FormAddUser, FormMessage, Message, User, UserPass};

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
        
        CREATE TABLE IF NOT EXISTS message
        (
            date INTEGER NOT NULL,
            room INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            
            UNIQUE(room, user_id, date),
            FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE
        );
        
        CREATE INDEX IF NOT EXISTS message_room
            on message (room);

        CREATE INDEX IF NOT EXISTS message_user_id
            on message (user_id);
        ",
    )?;

    return Ok(conn);
}

pub fn add_user<'a>(conn: &'a Connection, user: FormAddUser) -> Result<UserPass> {
    conn.execute(
        "INSERT INTO user (username, password, api_key) VALUES (?1,?2,?3)",
        (
            user.username.as_str(),
            bcrypt::hash(user.password.as_str()).unwrap(),
            "",
        ),
    )?;

    return Ok(UserPass {
        id: conn.last_insert_rowid(),
        username: user.username,
        pass: user.password,
        api_key: String::new(),
    });
}

pub fn logout<'a>(conn: &'a Connection, user_id: i64) -> Result<usize> {
    return user_update_api_key(conn, "", user_id);
}

pub fn user_select_id<'a>(conn: &'a Connection, user_id: i64) -> Result<UserPass, String> {
    let stmt = conn.prepare("SELECT id, username, password, api_key FROM user WHERE id = ?1");

    if stmt.is_err() {
        return Err(format!("cant prepare"));
    }

    let mut stmtmut = stmt.unwrap();

    let rows = stmtmut.query_map([user_id], map_user_pass);

    if rows.is_err() {
        return Err(format!("cant querry"));
    }

    let mut obduser = None;
    for usr in rows.unwrap() {
        if obduser.is_some() {
            return Err(format!("multiple users with the id {}", user_id));
        }
        if usr.is_err() {
            return Err(format!("bad user {}", usr.unwrap_err().to_string()));
        }
        obduser = Some(usr.unwrap());
    }
    if obduser.is_none() {
        return Err(format!("no user with the id {}", user_id));
    }
    return Ok(obduser.unwrap());
}

pub fn user_select_username<'a, 'b>(
    conn: &'a Connection,
    username: &'b str,
) -> Result<UserPass, String> {
    let stmt = conn.prepare("SELECT id, username, password, api_key FROM user WHERE username = ?1");

    if stmt.is_err() {
        return Err(format!("cant prepare"));
    }

    let mut stmtmut = stmt.unwrap();

    let rows = stmtmut.query_map([username], map_user_pass);

    if rows.is_err() {
        return Err(format!("cant querry"));
    }

    let mut obduser = None;
    for usr in rows.unwrap() {
        if obduser.is_some() {
            return Err(format!("multiple users with the username {}", username));
        }
        if usr.is_err() {
            return Err(format!("bad user {}", usr.unwrap_err().to_string()));
        }
        obduser = Some(usr.unwrap());
    }
    if obduser.is_none() {
        return Err(format!("no user with the username {}", username));
    }
    return Ok(obduser.unwrap());
}

pub fn user_update_api_key<'a, 'b>(
    conn: &'a Connection,
    api_key: &'b str,
    user_id: i64,
) -> Result<usize> {
    return conn.execute(
        "
        UPDATE user
        SET api_key = ?1
        WHERE id = ?2
        ",
        (api_key, user_id),
    );
}

fn map_user(row: &Row) -> Result<User> {
    return Ok(User {
        id: row.get(0)?,
        username: row.get(1)?,
    });
}

fn map_user_pass(row: &Row) -> Result<UserPass> {
    return Ok(UserPass {
        id: row.get(0)?,
        username: row.get(1)?,
        pass: row.get(2)?,
        api_key: row.get(3)?,
    });
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
