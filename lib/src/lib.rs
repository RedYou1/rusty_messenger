use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc};
use json::JsonValue;

pub fn serialize_login(username: String, password: String) -> HashMap<&'static str, String> {
    HashMap::<&'static str, String>::from([("username", username), ("password", password)])
}

pub fn serialize_message(
    room: i64,
    user_id: i64,
    api_key: String,
    text: String,
) -> HashMap<&'static str, String> {
    HashMap::<&'static str, String>::from([
        ("room_id", room.to_string()),
        ("user_id", user_id.to_string()),
        ("api_key", api_key),
        ("text", text),
    ])
}

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub id: i64,
    pub name: String,
}

impl Room {
    pub fn serialize(&self) -> String {
        format!(
            "{{ \"objectId\": {}, \"id\": {}, \"name\": \"{}\" }}",
            EventMessageId::Room.as_u8(),
            self.id,
            self.name.clone(),
        )
    }
}

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
            EventMessageId::Message.as_u8(),
            self.date.timestamp(),
            self.room_id,
            self.user_id,
            self.text.clone(),
        )
    }
}

enum EventMessageId {
    Room,
    Message,
}

impl EventMessageId {
    pub fn parse(value: u8) -> Option<EventMessageId> {
        match value {
            0 => Some(EventMessageId::Room),
            1 => Some(EventMessageId::Message),
            _ => None,
        }
    }
    pub fn as_u8(self) -> u8 {
        match self {
            EventMessageId::Room => 0,
            EventMessageId::Message => 1,
        }
    }
}

pub enum EventMessage {
    Room(Room),
    Message(Message),
}

impl EventMessage {
    pub fn parse<'a>(message: &'a JsonValue) -> Result<EventMessage, &'a str> {
        match message["objectId"]
            .as_u8()
            .map(|id| EventMessageId::parse(id))
        {
            Some(Some(EventMessageId::Room)) => Ok(EventMessage::Room(Room {
                id: message["id"]
                    .as_i64()
                    .ok_or("EventMessage Room.id Not found")?,
                name: message["name"]
                    .as_str()
                    .ok_or("EventMessage Room.name Not found")?
                    .to_string(),
            })),
            Some(Some(EventMessageId::Message)) => Ok(EventMessage::Message(Message {
                date: match Utc.timestamp_opt(
                    message["date"]
                        .as_i64()
                        .ok_or("EventMessage Message.date Not found")?,
                    0,
                ) {
                    chrono::LocalResult::Single(d) => Ok(d),
                    chrono::LocalResult::None => {
                        Err("EventMessage Message.date Error parsing date (None)")
                    }
                    chrono::LocalResult::Ambiguous(_, _) => {
                        Err("EventMessage Message.date Error parsing date (Ambiguous)")
                    }
                }?,
                room_id: message["room_id"]
                    .as_i64()
                    .ok_or("EventMessage Message.room_id Not found")?,
                user_id: message["user_id"]
                    .as_i64()
                    .ok_or("EventMessage Message.user_id Not found")?,
                text: message["text"]
                    .as_str()
                    .ok_or("EventMessage Message.text Not found")?
                    .to_string(),
            })),
            Some(None) => Err("EventMessage Object ID Not Supported"),
            None => Err("EventMessage Object ID Not Found"),
        }
    }
}
