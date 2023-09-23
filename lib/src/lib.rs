use chrono::{DateTime, Utc, TimeZone};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub date: DateTime<Utc>,
    pub room_id: i64,
    pub user_id: i64,
    pub text: String,
}

impl Message {
    pub fn serialize(&self) -> String {
        format!(
            "{{ \"objectId\": {}, \"date\": {}, \"room_id\": {}, \"user_id\": {}, \"text\": \"{}\" }}",
            0,
            self.date.timestamp(),
            self.room_id,
            self.user_id,
            self.text.clone(),
        )
    }

    pub fn deserialize(date: i64, room_id: i64, user_id: i64, text: &str) -> Message {
        Message {
            date: Utc.timestamp_opt(date, 0).unwrap(),
            room_id: room_id,
            user_id: user_id,
            text: text.to_string(),
        }
    }
}
