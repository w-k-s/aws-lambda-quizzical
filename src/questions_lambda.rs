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
    let page = event.get_query::<i64>("page").unwrap_or(1);
    let size = event.get_query::<i64>("size").unwrap_or(10);
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
