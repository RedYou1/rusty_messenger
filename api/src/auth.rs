use pwhash::bcrypt;
use rusqlite::Result;

use crate::{
    db::MyConnection,
    user::{new_api_key, new_api_key_2, AuthKey},
};

impl MyConnection {
    pub fn validate_user_with_api_key<'a, 'b>(
        &'a self,
        user_id: i64,
        api_key: &'b str,
    ) -> Result<String, String> {
        let bd_user = self.user_select_id(user_id)?;
        let bd_api_key = bd_user.api_key.as_str();

        if bd_api_key.eq("") || !bd_api_key.eq(api_key) {
            return Err(format!("bad user id or api key"));
        }

        let new_api_key = new_api_key_2(api_key);

        match self.user_update_api_key(new_api_key.as_str(), user_id) {
            Ok(_) => Ok(new_api_key),
            Err(_) => Err(format!("internal error while updating api key")),
        }
    }

    pub fn validate_login<'a, 'b, 'c>(
        &'a self,
        username: &'b str,
        password: &'c str,
    ) -> Result<AuthKey, String> {
        let bd_user = self.user_select_username(username)?;

        if !bcrypt::verify(password, bd_user.pass.as_str()) {
            return Err(format!("bad username or password"));
        }

        let new_api_key = new_api_key();

        match self.user_update_api_key(new_api_key.as_str(), bd_user.id) {
            Ok(_) => Ok(AuthKey {
                user_id: bd_user.id,
                api_key: new_api_key,
            }),
            Err(_) => Err(format!("internal error while updating api key")),
        }
    }
}
