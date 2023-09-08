use crate::room::Room;
use crate::User;
use crate::BASE_API_URL;

pub type AccountManager = Option<Account>;

#[derive(PartialEq)]
pub struct Account {
    _private: (),
    pub user: User,
}

impl Account {
    pub fn new(user: User) -> Account {
        Account {
            _private: (),
            user: user,
        }
    }

    pub async fn load_rooms(&self) -> (Result<Vec<Room>, String>, Option<String>) {
        let url = format!(
            "{BASE_API_URL}/room/all/{}?api_key={}",
            self.user.id, self.user.api_key
        );

        let res = reqwest::Client::new().get(&url).send().await;

        if let Err(e) = res {
            return (Err(e.to_string()), None);
        }

        let r = res.unwrap().text().await;

        if let Err(e) = r {
            return (Err(e.to_string()), None);
        }

        let value = json::parse(r.unwrap().as_str());

        if let Err(e) = value {
            return (Err(e.to_string()), None);
        }

        let value = value.unwrap();

        let mut napi_key: Option<String> = None;

        if let Some(api_key) = value["api_key"].as_str() {
            napi_key = Some(api_key.to_string());
        }

        if let Some(error) = value["reason"].as_str() {
            return (Err(error.to_string()), napi_key);
        }

        return (
            Ok(value["rooms"]
                .members()
                .map(|v| Room {
                    id: v["id"].as_i64().unwrap(),
                    name: v["name"].as_str().unwrap().to_string(),
                })
                .collect::<Vec<Room>>()),
            napi_key,
        );
    }
}
