use lib::{EventMessage, Message, Room};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, EventSource, MessageEvent};

use crate::{async_state::AsyncStateSetter, BASE_API_URL};

#[derive(PartialEq)]
pub enum SourceState {
    Error = 0b000,
    ReConnecting = 0b01,
    Connected = 0b10,
}

pub struct MyEventSource {
    source: EventSource,
    openF: Closure<dyn FnMut()>,
    errorF: Closure<dyn FnMut(Event)>,
    messageF: Closure<dyn FnMut(MessageEvent)>,
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
            openF: Closure::wrap(
                Box::new(move || open.set_state(SourceState::Connected)) as Box<dyn FnMut()>
            ),
            errorF: Closure::wrap(
                Box::new(move |_| error.set_state(SourceState::Error)) as Box<dyn FnMut(Event)>
            ),
            messageF: Closure::wrap(Box::new(move |event: MessageEvent| {
                let value = json::parse(
                    json::parse(event.data().as_string().unwrap().as_str())
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
                .unwrap();

                match EventMessage::parse(&value) {
                    Ok(EventMessage::Room(room)) => room_sender_thread.set_state(room),
                    Ok(EventMessage::Message(message)) => message_sender_thread.set_state(message),
                    Err(s) => panic!("{s}"),
                }
            }) as Box<dyn FnMut(MessageEvent)>),
        };

        source
            .source
            .set_onopen(Some(source.openF.as_ref().unchecked_ref()));
        source
            .source
            .set_onerror(Some(source.errorF.as_ref().unchecked_ref()));
        source
            .source
            .set_onmessage(Some(source.messageF.as_ref().unchecked_ref()));
        source
    }
}
