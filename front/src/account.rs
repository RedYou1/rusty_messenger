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

    pub async fn load_rooms(&mut self) -> Result<Vec<i64>, String> {
        let url = format!(
            "{BASE_API_URL}/rooms/all/{}?api_key={:?}",
            self.user.id, self.user.api_key
        );

        let res = reqwest::Client::new().get(&url).send().await;

        if let Err(e) = res {
            return Err(e.to_string());
        }

        let r = res.unwrap().text().await;

        if let Err(e) = r {
            return Err(e.to_string());
        }

        let value = json::parse(r.unwrap().as_str());

        if let Err(e) = value {
            return Err(e.to_string());
        }

        let value = value.unwrap();

        if let Some(api_key) = value["api_key"].as_str() {
            self.user.api_key = api_key.to_string();
        }

        if let Some(error) = value["reason"].as_str() {
            return Err(error.to_string());
        }

        return Ok(value["rooms"]
            .members()
            .map(|v| v.as_i64().unwrap())
            .collect::<Vec<i64>>());
    }
}
