use lib::{Message, Room};

use crate::{
    async_state::AsyncStateSetter,
    event_source::{MyEventSource, SourceState},
    structs::User,
};

pub struct AccountManager {
    current_user: Option<User>,
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
            current_user: None,
            try_tentative: 0,
            event_source: None,
            message_sender: message_sender,
            room_sender: room_sender,
            source_state_sender: source_state_sender,
        }
    }

    pub fn current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }

    pub fn set_current_user(&mut self, user: Option<User>) {
        match self.event_source.as_ref() {
            Some(e) => {
                e.close();
                self.event_source = None
            }
            None => {}
        }

        self.current_user = user;
        match self.current_user.as_ref() {
            Some(current_user) => {
                self.try_tentative = 1;
                self.event_source = Some(MyEventSource::new(
                    current_user.id,
                    current_user.api_key.as_str(),
                    &self.message_sender,
                    &self.room_sender,
                    &self.source_state_sender,
                ))
            }
            None => self.try_tentative = 0,
        }
    }

    pub fn retry_connection(&mut self) {
        self.try_tentative += 1;
        let current_user = self.current_user().unwrap();
        self.event_source.as_ref().unwrap().close();
        if self.try_tentative > 3 {
            self.current_user = None;
            self.event_source = None;
        } else {
            self.event_source = Some(MyEventSource::new(
                current_user.id,
                current_user.api_key.as_str(),
                &self.message_sender,
                &self.room_sender,
                &self.source_state_sender,
            ));
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.current_user.as_mut().unwrap().api_key = api_key;
    }

    pub fn set_connected(&mut self) {
        self.try_tentative = 1;
    }
}
