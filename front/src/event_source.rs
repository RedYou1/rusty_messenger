use sse_client::EventSource;

use crate::{
    async_state::AsyncStateSetter,
    room::Room,
    structs::{deserialize, Message},
    BASE_API_URL,
};

#[derive(PartialEq)]
pub enum SourceState {
    Disconnected = 0,
    ReConnecting = 1,
    Connected = 2,
}

pub struct MyEventSource {
    source: EventSource,
}

impl MyEventSource {
    pub fn new(
        user_id: i64,
        api_key: &str,
        message_sender: &AsyncStateSetter<Message>,
        room_sender: &AsyncStateSetter<Room>,
        source_state_sender: &AsyncStateSetter<SourceState>,
    ) -> MyEventSource {
        source_state_sender.set_state(SourceState::ReConnecting);

        let message_url = format!("{BASE_API_URL}/events/{}?api_key={}", user_id, api_key);

        let event_source = MyEventSource {
            source: EventSource::new(message_url.as_str()).unwrap(),
        };

        let open = source_state_sender.clone();
        event_source
            .source
            .on_open(move || open.set_state(SourceState::Connected));

        let close = source_state_sender.clone();
        event_source
            .source
            .add_event_listener("error", move |_| close.set_state(SourceState::Disconnected));

        let message_sender_thread = message_sender.clone();
        let room_sender_thread = room_sender.clone();
        event_source.source.on_message(move |event| {
            let value =
                json::parse(json::parse(event.data.as_str()).unwrap().as_str().unwrap()).unwrap();

            match value["objectId"].as_i8().unwrap() {
                0 => message_sender_thread.set_state(deserialize(
                    value["date"].as_i64().unwrap(),
                    value["room_id"].as_i64().unwrap(),
                    value["user_id"].as_i64().unwrap(),
                    value["text"].as_str().unwrap(),
                )),
                1 => room_sender_thread.set_state(Room {
                    id: value["id"].as_i64().unwrap(),
                    name: value["name"].as_str().unwrap().to_string(),
                }),
                _ => panic!("MyEventSource Object ID Not Supported"),
            }
        });

        return event_source;
    }
}
