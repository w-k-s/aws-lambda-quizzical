mod apigateway;
mod connection;
mod models;
mod repositories;
mod responses;

extern crate lambda_runtime as lambda;
extern crate log;
extern crate postgres;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

use apigateway::*;
use connection::connect_db_using_env_var;
use lambda::{error::HandlerError, lambda, Context};
use repositories::QuestionsRepository;
use responses::PaginatedResponse;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(questions_handler);
    Ok(())
}

fn questions_handler(
    event: APIGatewayEvent,
    c: Context,
) -> Result<APIGatewayResponse, HandlerError> {
    let page = event.get_query::<i64>("page").unwrap_or(1);
    let size = event.get_query::<i64>("size").unwrap_or(10);
    let category = event
        .get_query::<String>("category")
        .ok_or(c.new_error("Invalid Category"))?;

    let conn = connect_db_using_env_var("CONN_STRING")
        .map_err(|e| c.new_error(&format!("Connection Error: {}", e)))?;

    let (questions, total) = QuestionsRepository { conn: conn }
        .get_questions(&category, page, size)
        .map_err(|e| c.new_error(&format!("Error on get_questions: {}", e)))?;

    let paginated_response =
        PaginatedResponse::new(questions, page as u32, total as u32, size as u32);

    let api_response = APIGatewayResponse::new(200, &paginated_response).unwrap();
    Ok(api_response)
}
