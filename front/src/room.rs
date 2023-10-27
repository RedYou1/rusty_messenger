use std::fmt::Display;

use dioxus::prelude::Props;

use dioxus_router::routable::FromQuery;
use lib::Message;

#[derive(Debug)]
pub struct RoomData {
    pub name: String,
    pub messages: Vec<Message>,
}

#[derive(Clone, PartialEq, Props)]
pub struct OpRoomId {
    room_id: Option<Option<i64>>,
}

impl Into<Option<Option<i64>>> for OpRoomId {
    fn into(self) -> Option<Option<i64>> {
        self.room_id
    }
}

impl OpRoomId {
    pub fn new_empty() -> Self {
        OpRoomId { room_id: None }
    }
}

impl AsRef<Option<Option<i64>>> for OpRoomId {
    fn as_ref(&self) -> &Option<Option<i64>> {
        &self.room_id
    }
}

impl From<i64> for OpRoomId {
    fn from(value: i64) -> Self {
        OpRoomId {
            room_id: Some(Some(value)),
        }
    }
}

impl From<Option<i64>> for OpRoomId {
    fn from(value: Option<i64>) -> Self {
        OpRoomId {
            room_id: Some(value),
        }
    }
}

impl From<Option<Option<i64>>> for OpRoomId {
    fn from(value: Option<Option<i64>>) -> Self {
        OpRoomId { room_id: value }
    }
}

impl Display for OpRoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.room_id {
            Some(Some(id)) => write!(f, "{}", id),
            Some(None) => write!(f, "Error"),
            None => write!(f, ""),
        }
    }
}

impl FromQuery for OpRoomId {
    fn from_query(query: &str) -> Self {
        match query {
            "" => OpRoomId { room_id: None },
            "Error" => OpRoomId {
                room_id: Some(None),
            },
            _ => OpRoomId {
                room_id: Some(query.parse::<i64>().ok()),
            },
        }
    }
}
