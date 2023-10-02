use dioxus::prelude::Props;

use lib::Message;

#[derive(Debug)]
pub struct RoomData {
    pub name: String,
    pub messages: Vec<Message>,
}

#[derive(PartialEq, Props)]
pub struct OpRoomId {
    pub id: Option<i64>,
}
