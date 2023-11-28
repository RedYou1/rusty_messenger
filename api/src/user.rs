use chrono::Utc;
use pwhash::bcrypt;
use rand::Rng;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::{Result, Row};

use crate::database::Database;

/// Generate a new api_key (use new_api_key_2 when already have an api_key)
pub fn new_api_key() -> String {
    bcrypt::hash(format!(
        "{}+{}",
        Utc::now().timestamp(),
        rand::thread_rng().gen::<u64>()
    ))
    .unwrap()
}

/// Regenerate an api_key
pub fn new_api_key_2(previous: &str) -> String {
    bcrypt::hash(format!(
        "{}+{}+{}",
        Utc::now().timestamp(),
        previous,
        rand::thread_rng().gen::<u64>()
    ))
    .unwrap()
}

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
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct AuthKey {
    pub user_id: i64,
    pub api_key: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct AuthPass {
    pub user_id: i64,
    pub password: String,
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
pub struct FormAddUser {
    pub username: String,
    pub password: String,
}

impl Database {
    /// Crée un utilisateur et lui crée une api_key
    pub fn ajout_user(&self, user: FormAddUser) -> Result<AuthKey> {
        let napi = new_api_key();

        self.connection.execute(
            "INSERT INTO user (username, password, api_key) VALUES (?1,?2,?3)",
            (
                user.username.as_str(),
                bcrypt::hash(user.password.as_str()).unwrap(),
                napi.as_str(),
            ),
        )?;

        Ok(AuthKey {
            user_id: self.connection.last_insert_rowid(),
            api_key: napi,
        })
    }

    /// Supprime l'api_key d'un utilisateur
    pub fn logout(&self, user_id: i64) -> Result<usize> {
        self.user_update_api_key("", user_id)
    }

    /// Récupère tous les informations d'un utilisateur
    pub fn user_select_id(&self, user_id: i64) -> Result<UserPass, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, username, password, api_key FROM user WHERE id = ?1")
            .map_err(|_| String::from("cant prepare the querry"))?;

        let mut rows = stmt
            .query_map([user_id], map_user_pass)
            .map_err(|_| String::from("cant execute the querry"))?;

        match rows.next() {
            Some(Ok(bd_user)) => Ok(bd_user),
            _ => Err(format!("no user with the id {}", user_id)),
        }
    }

    /// Récupère le nom d'un utilisateur
    pub fn user_select_username(&self, username: &str) -> Result<UserPass, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, username, password, api_key FROM user WHERE username = ?1")
            .map_err(|_| String::from("cant prepare the querry"))?;

        let mut rows = stmt
            .query_map([username], map_user_pass)
            .map_err(|_| String::from("cant execute the querry"))?;

        match rows.next() {
            Some(Ok(bd_user)) => Ok(bd_user),
            _ => Err(format!("Pas d'utilisateur avec ce nom {}", username)),
        }
    }

    /// Change l'api_key d'un utilisateur
    pub fn user_update_api_key(&self, api_key: &str, user_id: i64) -> Result<usize> {
        self.connection.execute(
            "
        UPDATE user
        SET api_key = ?1
        WHERE id = ?2
        ",
            (api_key, user_id),
        )
    }
}

fn map_user_pass(row: &Row) -> Result<UserPass> {
    Ok(UserPass {
        id: row.get(0)?,
        username: row.get(1)?,
        pass: row.get(2)?,
        api_key: row.get(3)?,
    })
}
