mod apigateway;
mod connection;
mod models;
mod repositories;

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
use repositories::CategoriesRepository;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(categories_handler);
    Ok(())
}

fn categories_handler(
    _event: APIGatewayEvent,
    c: Context,
) -> Result<APIGatewayResponse, HandlerError> {
    let conn = connect_db_using_env_var("CONN_STRING")
        .map_err(|e| c.new_error(&format!("Connection Error: {}", e)))?;

    let categories = CategoriesRepository { conn: conn }
        .list_categories()
        .map_err(|e| c.new_error(&format!("{}", e)))?;

    let api_response = APIGatewayResponse::new(200, &categories).unwrap();
    Ok(api_response)
}
