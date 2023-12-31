//! Simulation de source d'événements pour les tests
//!
//! Ce module implémente une structure `TestEventSource` utilisée pour simuler une source
//! d'événements lors des tests, effectuant des requêtes HTTP pour récupérer des événements
//! et vérifiant s'ils correspondent aux événements attendus dans une application Rocket.

use json::parse;
use lib::EventMessage;
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::tokio::io::Lines;
use rocket::tokio::io::{AsyncBufReadExt, BufReader};

use crate::user::UserPass;

/// Pseudo Event Source pour les tests
#[derive(Debug)]
pub struct TestEventSource<'a> {
    username: String,
    stream: Lines<BufReader<LocalResponse<'a>>>,
}

impl<'a> TestEventSource<'a> {
    pub async fn new<'c>(
        client: &'c Client,
        user: &UserPass,
    ) -> Result<TestEventSource<'c>, String> {
        let response = client
            .get(format!(
                "/events/{}?api_key={}",
                user.id,
                user.api_key.as_str()
            ))
            .dispatch()
            .await;
    
        match response.status().code {
            200 => Ok(TestEventSource {
                username: user.username.to_string(),
                stream: BufReader::new(response).lines(),
            }),
            _ => Err(response.into_string().await.unwrap()),
        }
    }

    async fn next(&mut self) -> Result<Option<EventMessage>, String> {
        let mut line: Option<String> = None;
        for _ in 0..5 {
            line = match self.stream.next_line().await {
                Ok(Some(line)) if line.starts_with("data:") => Some(line),
                Ok(_) => continue,
                _ => return Err(String::from("Error Next Line")),
            };
            break;
        }
        match line {
            Some(line) => {
                let value = parse(&line[5..]).unwrap();

                match EventMessage::parse(&value) {
                    Ok(message) => Ok(Some(message)),
                    Err(message) => Err(String::from(message)),
                }
            }
            None => Ok(None),
        }
    }

    pub async fn test_next(&mut self, event: EventMessage) {
        match (self.next().await, event) {
            (Ok(Some(EventMessage::Message(message))), EventMessage::Message(event)) => {
                assert_eq!(message.user_id, event.user_id);
                assert_eq!(message.room_id, event.room_id);
                assert_eq!(message.text, event.text);
            }
            (Ok(Some(EventMessage::Message(message))), event) => {
                panic!(
                    "{}: Didn't expected a message: {:?} for event: {:?}",
                    self.username, message, event
                );
            }
            (Ok(Some(EventMessage::Room(room))), EventMessage::Room(event)) => {
                assert_eq!(room.id, event.id);
                assert_eq!(room.name, event.name);
            }
            (Ok(Some(EventMessage::Room(room))), event) => {
                panic!(
                    "{}: Didn't expected a room: {:?} for event: {:?}",
                    self.username, room, event
                );
            }
            (Ok(None), event) => {
                panic!("{}: No message for event: {:?}", self.username, event);
            }
            (Err(message), event) => {
                panic!("{}: {} for event: {:?}", self.username, message, event);
            }
        }
    }
}
