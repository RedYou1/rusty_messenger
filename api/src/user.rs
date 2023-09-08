use pwhash::bcrypt;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result, Row};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Debug)]
pub struct UserPass {
    pub id: i64,
    pub username: String,
    pub pass: String,
    pub api_key: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthKey {
    pub user_id: i64,
    pub api_key: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AuthPass {
    pub user_id: i64,
    pub password: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FormAddUser {
    pub username: String,
    pub password: String,
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
