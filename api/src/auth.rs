use chrono::Utc;
use pwhash::bcrypt;
use rand::Rng;
use rusqlite::Result;

use crate::db::MyConnection;

impl MyConnection {
    pub fn validate_user_key<'a, 'b>(
        &'a self,
        user_id: i64,
        api_key: &'b str,
    ) -> Result<String, String> {
        let bduser = self.user_select_id(user_id)?;
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

        match self.user_update_api_key(napi.as_str(), user_id) {
            Ok(_) => Ok(napi),
            Err(_) => Err(format!("internal error while updating api key")),
        }
    }

    pub fn validate_login<'a, 'b, 'c>(
        &'a self,
        username: &'b str,
        pass: &'c str,
    ) -> Result<(i64, String), String> {
        let bduser = self.user_select_username(username)?;

        if !bcrypt::verify(pass, bduser.pass.as_str()) {
            return Err(format!("bad user id or api key"));
        }

        let napi = bcrypt::hash(format!(
            "{}+{}",
            Utc::now().timestamp(),
            rand::thread_rng().gen::<u64>()
        ))
        .unwrap();

        match self.user_update_api_key(napi.as_str(), bduser.id) {
            Ok(_) => Ok((bduser.id, napi)),
            Err(_) => Err(format!("internal error while updating api key")),
        }
    }

    pub fn validate_user_pass<'a, 'b>(
        &'a self,
        user_id: i64,
        pass: &'b str,
    ) -> Result<String, String> {
        let bduser = self.user_select_id(user_id)?;

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

        match self.user_update_api_key(napi.as_str(), user_id) {
            Ok(_) => Ok(napi),
            Err(_) => Err(format!("internal error while updating api key")),
        }
    }
}
