mod apigateway;
mod connection;
mod models;
mod repositories;
mod responses;

extern crate http;
extern crate lambda_runtime as lambda;
extern crate log;
extern crate postgres;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

use apigateway::*;
use connection::connect_db_with_conn_string;
use http::StatusCode;
use lambda::{start, Context};
use log::info;
use models::{Category, Question};
use repositories::{CategoriesRepository, QuestionsRepository};
use std::error::Error;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    start(
        |event: APIGatewayEvent, c: Context| lambda_adapter(event, c, &new_question_handler),
        None,
    );
    Ok(())
}

fn new_question_handler<'a>(
    event: APIGatewayEvent,
    config: Config,
) -> Result<APIGatewayResponse, APIErrorResponse> {
    let question: Question = match event.parse() {
        Ok(Some(question)) => question,
        Ok(None) => {
            return Err(APIErrorResponse::new(
                StatusCode::BAD_REQUEST,
                "question required in body".into(),
            ))
        }
        Err(e) => {
            return Err(APIErrorResponse::new(
                StatusCode::BAD_REQUEST,
                format!("{}", e),
            ))
        }
    };

    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let categories_repository = CategoriesRepository { conn: conn.clone() };
    let _ = categories_repository.save_category(&Category {
        title: question.category.clone(),
    });

    let question_repository = QuestionsRepository { conn: conn.clone() };
    let new_question = question_repository.save_question(&question)?;

    let api_response = APIGatewayResponse::new(StatusCode::CREATED, Some(&new_question)).unwrap();
    Ok(api_response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_body_returns_400() {
        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match new_question_handler(event, config) {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.status_code(), StatusCode::BAD_REQUEST),
        }
    }

    #[test]
    fn test_invalid_json_returns_400() {
        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: Some("{}".into()),
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match new_question_handler(event, config) {
            Ok(_) => assert!(false),
            Err(err) => {
                print!("TEST. Invalid json. Error: '{}'\n", err.message());
                assert_eq!(err.status_code(), StatusCode::BAD_REQUEST)
            }
        }
    }

    #[test]
    fn test_save_question() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        std::env::set_var("CONN_STRING", std::env::var("TEST_CONN_STRING").unwrap());

        let question_json = r#"{
            "question": "Why did the chicken cross the road",
            "category": "Joke",
            "choices":[{
                "title":"To get to the other side",
                "correct":true
            },{
                "title":"To commit suicide",
                "correct":false
            }]
        }"#;

        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: Some(question_json.into()),
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match new_question_handler(event, config) {
            Ok(apiresponse) => {
                let question: Question = apiresponse.parse().unwrap();
                let choice = question.choices.first().unwrap();

                assert_eq!(apiresponse.status_code, StatusCode::CREATED);
                assert!(question.id.is_some());
                assert_eq!(
                    question.question,
                    "Why did the chicken cross the road".to_string()
                );
                assert_eq!(question.category, "Joke".to_string());
                assert_eq!(question.choices.len(), 2);
                assert!(choice.id.is_some());
                assert_eq!(choice.title, "To get to the other side".to_string());
                assert!(choice.correct);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_question_with_multiple_correct_options_returns_400() {
        std::env::set_var("CONN_STRING", std::env::var("TEST_CONN_STRING").unwrap());

        let question_json = r#"{
            "question": "Why did the chicken cross the road",
            "category": "Joke",
            "choices":[{
                "title":"To get to the other side",
                "correct":true
            },{
                "title":"To commit suicide",
                "correct":true
            }]
        }"#;

        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: Some(question_json.into()),
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match new_question_handler(event, config) {
            Ok(_) => assert!(false),
            Err((status_code, msg)) => {
                print!("TEST. Invalid json. Error: '{}'\n", msg);
                assert_eq!(status_code, StatusCode::BAD_REQUEST)
            }
        }
    }
}
