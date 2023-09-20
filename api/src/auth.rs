use chrono::Utc;
use pwhash::bcrypt;
use rand::Rng;
use rusqlite::{Connection, Result};

use crate::user::{user_select_id, user_select_username, user_update_api_key};

pub fn validate_user_key<'a, 'b>(
    conn: &'a Connection,
    user_id: i64,
    api_key: &'b str,
) -> Result<String, String> {
    let bduser = user_select_id(conn, user_id)?;
    let bdapi_key = bduser.api_key.as_str();

    if bdapi_key.eq("") {
        return Err(format!("bad user id or api key"));
    }

    if !bdapi_key.eq(api_key) {
        return Err(format!("bad user id or api key"));
    }

    let napi = bcrypt::hash(format!(
        "{}+{}+{}",
        Utc::now().timestamp(),
        api_key,
        rand::thread_rng().gen::<u64>()
    ))
    .unwrap();

    if user_update_api_key(&conn, napi.as_str(), user_id).is_err() {
        return Err(format!("internal error while updating api key"));
    }

    return Ok(napi);
}

pub fn validate_login<'a, 'b, 'c>(
    conn: &'a Connection,
    username: &'b str,
    pass: &'c str,
) -> Result<(i64, String), String> {
    let bduser = user_select_username(conn, username)?;

    if !bcrypt::verify(pass, bduser.pass.as_str()) {
        return Err(format!("bad user id or api key"));
    }

    let napi = bcrypt::hash(format!(
        "{}+{}",
        Utc::now().timestamp(),
        rand::thread_rng().gen::<u64>()
    ))
    .unwrap();

    if user_update_api_key(&conn, napi.as_str(), bduser.id).is_err() {
        return Err(format!("internal error while updating api key"));
    }

    return Ok((bduser.id, napi));
}

pub fn validate_user_pass<'a, 'b>(
    conn: &'a Connection,
    user_id: i64,
    pass: &'b str,
) -> Result<String, String> {
    let bduser = user_select_id(conn, user_id)?;

    if !bcrypt::verify(bduser.pass.as_str(), pass) {
        return Err(format!("bad user id or api key"));
    }

    let napi = bcrypt::hash(format!(
        "{}+{}+{}",
        Utc::now().timestamp(),
        bduser.api_key,
        rand::thread_rng().gen::<u64>()
    ))
    .unwrap();

    if user_update_api_key(&conn, napi.as_str(), user_id).is_err() {
        return Err(format!("internal error while updating api key"));
    }

    return Ok(napi);
}
