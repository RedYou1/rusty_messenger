use lib::Message;

use crate::{
    async_state::AsyncStateSetter,
    event_source::{MyEventSource, SourceState},
    room::Room,
    structs::User,
};

pub struct AccountManager {
    current: Option<User>,
    try_tentative: i64,
    event_source: Option<MyEventSource>,
    message_sender: AsyncStateSetter<Message>,
    room_sender: AsyncStateSetter<Room>,
    source_state_sender: AsyncStateSetter<SourceState>,
}

impl AccountManager {
    pub fn new(
        message_sender: AsyncStateSetter<Message>,
        room_sender: AsyncStateSetter<Room>,
        source_state_sender: AsyncStateSetter<SourceState>,
    ) -> AccountManager {
        AccountManager {
            current: None,
            try_tentative: 0,
            event_source: None,
            message_sender: message_sender,
            room_sender: room_sender,
            source_state_sender: source_state_sender,
        }
    }

    pub fn user(&self) -> Option<&User> {
        self.current.as_ref()
    }

    pub fn set_user(&mut self, user: Option<User>) {
        match self.event_source.as_ref() {
            Some(e) => {
                e.close();
                self.event_source = None
            }
            None => {}
        }

        self.current = user;
        match self.current.as_ref() {
            Some(e) => {
                self.try_tentative = 1;
                self.event_source = Some(MyEventSource::new(
                    e.id,
                    e.api_key.as_str(),
                    &self.message_sender,
                    &self.room_sender,
                    &self.source_state_sender,
                ))
            }
            None => self.try_tentative = 0,
        }
    }

    pub fn retry(&mut self) {
        self.try_tentative += 1;
        let user = self.user().unwrap();
        self.event_source.as_ref().unwrap().close();
        if self.try_tentative > 3 {
            self.current = None;
            self.event_source = None;
        } else {
            self.event_source = Some(MyEventSource::new(
                user.id,
                user.api_key.as_str(),
                &self.message_sender,
                &self.room_sender,
                &self.source_state_sender,
            ));
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.current.as_mut().unwrap().api_key = api_key;
    }

    pub fn connected(&mut self) {
        self.try_tentative = 1;
    }
}
