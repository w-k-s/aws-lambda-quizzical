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
use repositories::QuestionsRepository;
use responses::PaginatedResponse;
use std::error::Error;
use std::sync::Arc;

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_SIZE: i64 = 10;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    start(
        |event: APIGatewayEvent, c: Context| lambda_adapter(event, c, &questions_handler),
        None,
    );
    Ok(())
}

fn questions_handler<'a>(
    event: APIGatewayEvent,
    config: Config,
) -> Result<APIGatewayResponse, APIError> {
    let page = match event.get_query::<i64>("page") {
        Some(x) if x >= DEFAULT_PAGE => x,
        _ => DEFAULT_PAGE,
    };
    let size = match event.get_query::<i64>("size") {
        Some(x) if x >= DEFAULT_SIZE => x,
        _ => DEFAULT_SIZE,
    };
    let category = event.get_query::<String>("category").ok_or((
        StatusCode::BAD_REQUEST,
        APIErrorResponse {
            message: "Invalid Category".to_owned(),
        },
    ))?;

    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let repository = QuestionsRepository { conn: conn };
    let total = repository.count_questions(&category)?;
    let questions = match total {
        0 => vec![],
        _ => repository.get_questions(&category, page, size)?,
    };

    let paginated_response =
        PaginatedResponse::new(questions, page as u32, total as u32, size as u32);

    let api_response = APIGatewayResponse::new(200, &paginated_response).unwrap();
    Ok(api_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{Category, Question};
    use repositories::CategoriesRepository;
    use std::collections::HashMap;
    #[test]
    fn test_empty_query_returns_400() {
        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match questions_handler(event, config) {
            Ok(_) => assert!(false),
            Err((status_code, _)) => {
                assert_eq!(status_code, 400);
            }
        }
    }

    #[test]
    fn test_empty_page_and_size_uses_defaults() {
        let mut query = HashMap::<String, String>::new();
        query.insert("category".into(), "Joke".into());

        let event = APIGatewayEvent {
            path: "/".into(),
            query: Some(query),
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match questions_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code, 200);

                let paginated_response: PaginatedResponse<Question> = resp.parse().unwrap();
                assert_eq!(paginated_response.page, DEFAULT_PAGE as u32);
                assert!(paginated_response.size <= DEFAULT_SIZE as u32);
            }
        }
    }

    #[test]
    fn test_zero_values_uses_defaults() {
        let mut query = HashMap::<String, String>::new();
        query.insert("category".into(), "Joke".into());
        query.insert("page".into(), "0".into());
        query.insert("size".into(), "0".into());

        let event = APIGatewayEvent {
            path: "/".into(),
            query: Some(query),
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match questions_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code, 200);

                let paginated_response: PaginatedResponse<Question> = resp.parse().unwrap();
                assert_eq!(paginated_response.page, DEFAULT_PAGE as u32);
                assert!(paginated_response.size <= DEFAULT_SIZE as u32);
            }
        }
    }

    #[test]
    fn test_negative_values_uses_defaults() {
        let mut query = HashMap::<String, String>::new();
        query.insert("category".into(), "Joke".into());
        query.insert("page".into(), "0".into());
        query.insert("size".into(), "0".into());

        let event = APIGatewayEvent {
            path: "/".into(),
            query: Some(query),
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match questions_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code, 200);

                let paginated_response: PaginatedResponse<Question> = resp.parse().unwrap();
                assert_eq!(paginated_response.page, DEFAULT_PAGE as u32);
                assert!(paginated_response.size <= DEFAULT_SIZE as u32);
            }
        }
    }

    #[test]
    fn test_alphabetic_data_uses_defaults() {
        let mut query = HashMap::<String, String>::new();
        query.insert("category".into(), "Joke".into());
        query.insert("page".into(), "PAGE".into());
        query.insert("size".into(), "SIZE".into());

        let event = APIGatewayEvent {
            path: "/".into(),
            query: Some(query),
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        match questions_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code, 200);

                let paginated_response: PaginatedResponse<Question> = resp.parse().unwrap();
                assert_eq!(paginated_response.page, DEFAULT_PAGE as u32);
                assert!(paginated_response.size <= DEFAULT_SIZE as u32);
            }
        }
    }

    fn populate_db() {
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

        let question: Question = serde_json::from_str(&question_json).unwrap();

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        let conn = Arc::new(connect_db_with_conn_string(&config.connection_string).unwrap());

        let categories_repository = CategoriesRepository { conn: conn.clone() };
        let _ = categories_repository.save_category_and_set_active(
            &Category {
                title: question.category.clone(),
            },
            Some(true),
        );

        let questions_repository = QuestionsRepository { conn: conn.clone() };
        let _ = questions_repository.save_question(&question).unwrap();
    }

    #[test]
    fn test_load_questions_retuns_paginated_list() {
        populate_db();

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        let mut query = HashMap::<String, String>::new();
        query.insert("category".into(), "Joke".into());

        let event = APIGatewayEvent {
            path: "/".into(),
            query: Some(query),
            body: None,
        };

        match questions_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code, 200);

                let paginated_response: PaginatedResponse<Question> = resp.parse().unwrap();
                assert_eq!(paginated_response.page, DEFAULT_PAGE as u32);
                assert!(paginated_response.size <= DEFAULT_SIZE as u32);

                let questions = paginated_response.data;
                let first_question = questions.first().unwrap();
                let first_choice = first_question.choices.first().unwrap();

                assert!(questions.len() >= 1);

                assert_eq!(
                    first_question.question,
                    "Why did the chicken cross the road".to_string()
                );
                assert_eq!(first_question.category, "Joke".to_string());
                assert_eq!(first_question.choices.len(), 2);
                assert_eq!(first_choice.title, "To get to the other side".to_string());
                assert!(first_choice.correct);
            }
        }
    }
}
