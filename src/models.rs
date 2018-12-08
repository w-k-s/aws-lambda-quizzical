extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Categories {
    pub categories: Vec<Category>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choice {
    pub title: String,
    pub correct: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: i64,
    pub question: String,
    pub category: String,
    pub choices: Vec<Choice>,
}
