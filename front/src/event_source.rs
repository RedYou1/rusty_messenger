use std::sync::{Arc, Mutex};

use dioxus::prelude::Coroutine;
use sse_client::EventSource;

use crate::{
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
        message_sender: &Arc<Mutex<Coroutine<Message>>>,
        source_state_sender: &Arc<Mutex<Coroutine<SourceState>>>,
    ) -> MyEventSource {
        let init = Arc::clone(source_state_sender);
        set_source_state(&init, SourceState::ReConnecting);

        let url = format!("{BASE_API_URL}/events/{}?api_key={}", user_id, api_key);

        let event_source = MyEventSource {
            source: EventSource::new(url.as_str()).unwrap(),
        };

        let open = Arc::clone(source_state_sender);
        event_source.source.on_open(move || {
            set_source_state(&open, SourceState::Connected);
        });

        let close = Arc::clone(source_state_sender);
        event_source.source.add_event_listener("error", move |_| {
            set_source_state(&close, SourceState::Disconnected);
        });

        let sender_thread = Arc::clone(&message_sender);
        event_source.source.on_message(move |event| {
            let value = json::parse(event.data.as_str()).unwrap();
            let message = deserialize(
                value["date"].as_i64().unwrap(),
                value["room"].as_i64().unwrap(),
                value["user_id"].as_i64().unwrap(),
                value["text"].as_str().unwrap(),
            );
            let sender = sender_thread.lock().unwrap();
            sender.send(message);
        });

        return event_source;
    }
}

fn set_source_state(state_state: &Arc<Mutex<Coroutine<SourceState>>>, state: SourceState) {
    let s = state_state.as_ref().lock().unwrap();
    s.send(state);
}
