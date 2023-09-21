use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Event, EventSource, MessageEvent};

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
    openF: Closure<dyn FnMut()>,
    errorF: Closure<dyn FnMut(Event)>,
    messageF: Closure<dyn FnMut(MessageEvent)>,
}

impl MyEventSource {
    pub fn close(&self){
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
                Box::new(move |_| error.set_state(SourceState::Disconnected))
                    as Box<dyn FnMut(Event)>,
            ),
            messageF: Closure::wrap(Box::new(move |event: MessageEvent| {
                let value = json::parse(
                    json::parse(event.data().as_string().unwrap().as_str())
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )
                .unwrap();

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
        return source;
    }
}
