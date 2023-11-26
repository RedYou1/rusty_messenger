use dotenv::dotenv;
use rusqlite::{Connection, Result};
use std::env;
use std::sync::Once;

pub struct Database {
    _private: (),
    pub connection: Connection,
}

static mut DATABASE_URL: String = String::new();
static INIT: Once = Once::new();

fn database_url(is_unit_test: bool) -> &'static String {
    INIT.call_once(|| {
        dotenv().unwrap();
        let path = match is_unit_test {
            false => "DATABASE_URL",
            true => "DATABASE_URL_TEST",
        };
        let url = env::var(path).unwrap();
        unsafe {
            DATABASE_URL = url;
        }
    });
    unsafe { &DATABASE_URL }
}

impl Database {
    pub fn new(is_unit_test: bool) -> Result<Database> {
        Ok(Database {
            _private: (),
            connection: Connection::open(database_url(is_unit_test))?,
        })
    }

    pub fn create_tables(&self) -> Result<()> {
        self.connection.execute_batch(
            " 
            CREATE TABLE IF NOT EXISTS user
            (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                api_key TEXT NOT NULL
            );
    
            CREATE UNIQUE INDEX IF NOT EXISTS user_username
                on user (username);
    
            CREATE TABLE IF NOT EXISTS room
            (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );
    
            CREATE TABLE IF NOT EXISTS message
            (
                date INTEGER NOT NULL,
                room_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                text TEXT NOT NULL,
    
                FOREIGN KEY(user_id, room_id) REFERENCES user_room(user_id, room_id) ON DELETE CASCADE
            );
    
            CREATE TABLE IF NOT EXISTS user_room
            (
                user_id INTEGER NOT NULL,
                room_id INTEGER NOT NULL,
    
                PRIMARY KEY(user_id, room_id),
                FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE,
                FOREIGN KEY(room_id) REFERENCES room(id) ON DELETE CASCADE
            );
            ",
        )
    }
}
