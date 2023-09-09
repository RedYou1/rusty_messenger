use std::sync::{Arc, Mutex};

use dioxus::prelude::{use_coroutine, Coroutine, Scope, UseSharedState};
use futures_channel::mpsc::UnboundedReceiver;
use futures_lite::StreamExt;
use sse_client::EventSource;

use crate::{
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

pub type EventReceiver<Value> = Arc<Mutex<Coroutine<Value>>>;

pub fn event_receiver<Container, Value, Func>(
    cx: Scope,
    state: &UseSharedState<Container>,
    func: Func,
) -> EventReceiver<Value>
where
    Container: 'static,
    Value: 'static,
    Func: Fn(&UseSharedState<Container>, Value) + 'static,
{
    Arc::new(Mutex::new(
        use_coroutine(cx, |mut receiver: UnboundedReceiver<Value>| unsafe {
            let state = state as *const UseSharedState<Container>;
            async move {
                loop {
                    match receiver.next().await {
                        Some(value) => {
                            func(state.as_ref().unwrap(), value);
                        }
                        None => println!("None"),
                    }
                }
            }
        })
        .to_owned(),
    ))
}

impl MyEventSource {
    pub fn new(
        user_id: i64,
        api_key: &str,
        message_sender: &EventReceiver<Message>,
        room_sender: &EventReceiver<Room>,
        source_state_sender: &EventReceiver<SourceState>,
    ) -> MyEventSource {
        let init: EventReceiver<SourceState> = Arc::clone(source_state_sender);
        send_event(&init, SourceState::ReConnecting);

        let message_url = format!("{BASE_API_URL}/events/{}?api_key={}", user_id, api_key);

        let event_source = MyEventSource {
            source: EventSource::new(message_url.as_str()).unwrap(),
        };

        let open: EventReceiver<SourceState> = Arc::clone(source_state_sender);
        event_source.source.on_open(move || {
            send_event(&open, SourceState::Connected);
        });

        let close: EventReceiver<SourceState> = Arc::clone(source_state_sender);
        event_source.source.add_event_listener("error", move |_| {
            send_event(&close, SourceState::Disconnected);
        });

        let message_sender_thread: EventReceiver<Message> = Arc::clone(&message_sender);
        let room_sender_thread: EventReceiver<Room> = Arc::clone(&room_sender);
        event_source.source.on_message(move |event| {
            let value =
                json::parse(json::parse(event.data.as_str()).unwrap().as_str().unwrap()).unwrap();

            match value["objectId"].as_i8().unwrap() {
                0 => send_event(
                    &message_sender_thread,
                    deserialize(
                        value["date"].as_i64().unwrap(),
                        value["room_id"].as_i64().unwrap(),
                        value["user_id"].as_i64().unwrap(),
                        value["text"].as_str().unwrap(),
                    ),
                ),
                1 => send_event(
                    &room_sender_thread,
                    Room {
                        id: value["id"].as_i64().unwrap(),
                        name: value["name"].as_str().unwrap().to_string(),
                    },
                ),
                _ => panic!("MyEventSource Object ID Not Supported"),
            }
        });

        return event_source;
    }
}

fn send_event<Value>(sender: &EventReceiver<Value>, value: Value)
where
    Value: 'static,
{
    let s = sender.as_ref().lock().unwrap();
    s.send(value);
}
