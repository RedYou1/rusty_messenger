#![allow(non_snake_case)]

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}
