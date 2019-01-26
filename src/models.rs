extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    Constraint { pointer: String, message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ValidationError::Constraint { pointer, message } = self;
        write!(f, "ValidationError{{ {}: {} }}", pointer, message,)
    }
}

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
    pub id: Option<i64>,
    pub title: String,
    pub correct: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: Option<i64>,
    pub question: String,
    pub category: String,
    pub choices: Vec<Choice>,
}

impl Question {
    pub fn validate(question: &Question) -> Result<(), ValidationError> {
        if question
            .choices
            .iter()
            .filter(|choice| choice.correct)
            .count()
            > 1
        {
            return Err(ValidationError::Constraint {
                pointer: "/data/attribute/choices".into(),
                message: "Only one correct choice allowed".into(),
            });
        }
        Ok(())
    }
}
