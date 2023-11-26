use lib::{EventMessage, Message, Room};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, EventSource, MessageEvent};

use crate::{async_state::AsyncStateSetter, BASE_API_URL};

#[derive(Clone, Copy, PartialEq)]
pub enum SourceState {
    Error,
    ReConnecting,
    Connected,
}

pub struct MyEventSource {
    source: EventSource,
    open_function: Closure<dyn FnMut()>,
    error_function: Closure<dyn FnMut(Event)>,
    message_function: Closure<dyn FnMut(MessageEvent)>,
}

impl MyEventSource {
    pub fn close(&self) {
        self.source.close();
    }

    pub fn new(
        user_id: i64,
        api_key: &str,
        message_sender: &AsyncStateSetter<Message>,
        room_sender: &AsyncStateSetter<Room>,
        source_state_sender: &AsyncStateSetter<SourceState>,
    ) -> MyEventSource {
        source_state_sender.set_state(SourceState::ReConnecting);

        let message_url = format!("{BASE_API_URL}/events/{}?api_key={}", user_id, api_key);

        let open = source_state_sender.clone();
        let error = source_state_sender.clone();
        let message_sender_thread = message_sender.clone();
        let room_sender_thread = room_sender.clone();

        let source = MyEventSource {
            source: EventSource::new(message_url.as_str()).unwrap(),
            open_function: Closure::wrap(
                Box::new(move || open.set_state(SourceState::Connected)) as Box<dyn FnMut()>
            ),
            error_function: Closure::wrap(
                Box::new(move |_| error.set_state(SourceState::Error)) as Box<dyn FnMut(Event)>
            ),
            message_function: Closure::wrap(Box::new(move |event: MessageEvent| {
                let value = json::parse(event.data().as_string().unwrap().as_str()).unwrap();

                match EventMessage::parse(&value) {
                    Ok(EventMessage::Room(room)) => room_sender_thread.set_state(room),
                    Ok(EventMessage::Message(message)) => message_sender_thread.set_state(message),
                    Err(s) => panic!("{s}"),
                }
            }) as Box<dyn FnMut(MessageEvent)>),
        };

        source
            .source
            .set_onopen(Some(source.open_function.as_ref().unchecked_ref()));
        source
            .source
            .set_onerror(Some(source.error_function.as_ref().unchecked_ref()));
        source
            .source
            .set_onmessage(Some(source.message_function.as_ref().unchecked_ref()));
        source
    }
}
