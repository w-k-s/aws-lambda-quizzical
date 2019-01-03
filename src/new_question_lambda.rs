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
use connection::connect_db_using_env_var;
use http::StatusCode;
use log::{info};
use lambda::{start, Context};
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
) -> Result<APIGatewayResponse, APIError> {
    let body = event.body.ok_or((
        StatusCode::BAD_REQUEST,
        APIErrorResponse {
            message: "question required in body".into(),
        },
    ))?;

    let question: Question = serde_json::from_str(&body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            APIErrorResponse {
                message: format!("{}", e),
            },
        )
    })?;

    let conn = Arc::new(connect_db_using_env_var("CONN_STRING")?);

    let categories_repository = CategoriesRepository { conn: conn.clone() };
    categories_repository.save_category(&Category {
        title: question.category.clone(),
    });

    let question_repository = QuestionsRepository { conn: conn.clone() };
    let new_question = question_repository.save_question(&question)?;

    let api_response = APIGatewayResponse::new(201, &new_question).unwrap();
    Ok(api_response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_body_returns_400(){
        let event = APIGatewayEvent{
            path: "/".into(),
            query: None,
            body: None,
        };
        
        match new_question_handler(event){
            Ok(_) => assert!(false),
            Err((status_code, _)) => assert_eq!(status_code, StatusCode::BAD_REQUEST)
        }
    }

    #[test]
    fn test_invalid_json_returns_400(){

        let event = APIGatewayEvent{
            path: "/".into(),
            query: None,
            body: Some("{}".into()),
        };
        
        match new_question_handler(event){
            Ok(_) => assert!(false),
            Err((status_code, msg)) => {
                print!("TEST. Invalid json. Error: '{}'\n",msg);
                assert_eq!(status_code, StatusCode::BAD_REQUEST)
            }
        }
    }

    #[test]
    fn test_save_question(){
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

        let event = APIGatewayEvent{
            path: "/".into(),
            query: None,
            body: Some(question_json.into()),
        };
        
        match new_question_handler(event){
            Ok(apiresponse) => {
                let question: Question = serde_json::from_str(&apiresponse.body).unwrap();
                assert_eq!(apiresponse.status_code, 201);
                assert!(question.id.is_some());
                assert_eq!(question.question, "Why did the chicken cross the road".to_string());
                assert_eq!(question.category, "Joke".to_string());
                assert_eq!(question.choices.len(), 2);
                assert_eq!(question.choices.first().unwrap().title, "To get to the other side".to_string());
                assert!(question.choices.first().unwrap().correct);
            },
            Err(_) => assert!(false), 
        }
    }
}