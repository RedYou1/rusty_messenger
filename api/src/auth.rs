use pwhash::bcrypt;
use rusqlite::Result;

use crate::{
    database::Database,
    user::{new_api_key, new_api_key_2, AuthKey},
};

impl Database {
    pub fn validate_user_with_api_key(
        &self,
        user_id: i64,
        api_key: &str,
    ) -> Result<String, String> {
        let bd_user = self.user_select_id(user_id)?;
        let bd_api_key = bd_user.api_key.as_str();

        if bd_api_key.eq("") || !bd_api_key.eq(api_key) {
            return Err(String::from("Mauvais id ou api key"));
        }

        let new_api_key = new_api_key_2(api_key);

        match self.user_update_api_key(new_api_key.as_str(), user_id) {
            Ok(_) => Ok(new_api_key),
            Err(_) => Err(String::from("internal error while updating api key")),
        }
    }

    pub fn validate_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthKey, String> {
        let bd_user = self.user_select_username(username)?;

        if !bcrypt::verify(password, bd_user.pass.as_str()) {
            return Err(String::from("Mauvais identifiant ou mot de passe"));
        }

        let new_api_key = new_api_key();

        match self.user_update_api_key(new_api_key.as_str(), bd_user.id) {
            Ok(_) => Ok(AuthKey {
                user_id: bd_user.id,
                api_key: new_api_key,
            }),
            Err(_) => Err(String::from("internal error while updating api key")),
        }
    }
}
