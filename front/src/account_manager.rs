use lib::{Message, Room};

use crate::{
    async_state::AsyncStateSetter,
    event_source::{MyEventSource, SourceState},
    structs::User,
};

/// Gère le système d'authentification de l'utilisateur
pub struct AccountManager {
    utilisateur_actuelle: Option<User>,
    nombre_tentatives: i64,
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
            utilisateur_actuelle: None,
            nombre_tentatives: 0,
            event_source: None,
            message_sender: message_sender,
            room_sender: room_sender,
            source_state_sender: source_state_sender,
        }
    }

    pub fn utilisateur_actuelle(&self) -> Option<&User> {
        self.utilisateur_actuelle.as_ref()
    }

    pub fn modifier_utilisateur_actuelle(&mut self, user: Option<User>) {
        match self.event_source.as_ref() {
            Some(e) => {
                e.close();
                self.event_source = None
            }
            None => {}
        }

        self.utilisateur_actuelle = user;
        match self.utilisateur_actuelle.as_ref() {
            Some(current_user) => {
                self.nombre_tentatives = 1;
                self.event_source = Some(MyEventSource::new(
                    current_user.id,
                    current_user.api_key.as_str(),
                    &self.message_sender,
                    &self.room_sender,
                    &self.source_state_sender,
                ))
            }
            None => self.nombre_tentatives = 0,
        }
    }

    pub fn nouvelle_tentative_de_connection(&mut self) {
        self.nombre_tentatives += 1;
        let current_user = self.utilisateur_actuelle().unwrap();
        self.event_source.as_ref().unwrap().close();
        if self.nombre_tentatives > 3 {
            self.utilisateur_actuelle = None;
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

    pub fn modifier_api_key(&mut self, api_key: String) {
        self.utilisateur_actuelle.as_mut().unwrap().api_key = api_key;
    }

    pub fn Mettre_est_connecter(&mut self) {
        self.nombre_tentatives = 1;
    }
}
