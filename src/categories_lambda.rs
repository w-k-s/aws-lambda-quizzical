mod apigateway;
mod connection;
mod models;
mod repositories;

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
use lambda::{start, Context};
use repositories::CategoriesRepository;
use std::error::Error;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    start(
        |event: APIGatewayEvent, c: Context| lambda_adapter(event, c, &categories_handler),
        None,
    );
    Ok(())
}

fn categories_handler(
    _event: APIGatewayEvent,
    config: Config,
) -> Result<APIGatewayResponse, APIError> {
    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let categories = CategoriesRepository { conn: conn }.list_categories()?;
    let api_response = APIGatewayResponse::new(200, &categories).unwrap();

    Ok(api_response)
}
