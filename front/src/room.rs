use dioxus::prelude::Props;

#[derive(Debug)]
pub struct Room {
    pub id: i64,
    pub name: String,
}

#[derive(PartialEq, Props)]
pub struct OpRoomId {
    pub id: Option<i64>,
}
