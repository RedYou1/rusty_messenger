//! Module  qui implémente le struct User.
//! 
//! Il contient son id, nom et mot de passe.

#![allow(non_snake_case)]

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub api_key: String,
}
