mod apigateway;

extern crate lambda_runtime as lambda;
extern crate log;
extern crate postgres;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate simple_logger;

use apigateway::*;
use lambda::{error::HandlerError, lambda, Context};
use log::{error, info};
use postgres::{Connection, TlsMode};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct Category {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct CategoriesResponse {
    categories: Vec<Category>,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);
    Ok(())
}

fn my_handler(event: APIGatewayEvent, c: Context) -> Result<APIGatewayResponse, HandlerError> {
    let conn_string_var =
        env::var_os("CONN_STRING").map(|host| host.into_string().expect("invalid CONN_STRING"));

    let conn_string = match conn_string_var {
        Some(var) => var,
        None => return Err(c.new_error("env CONN_STRING not set")),
    };

    let conn = Connection::connect(conn_string, TlsMode::None)
        .map_err(|e| c.new_error(&format!("Connection Error: {}", e)))?;

    let rows = &conn
        .query("SELECT name FROM categories", &[])
        .map_err(|e| c.new_error(&format!("Query Error: {}", e)))?;

    let mut categories: Vec<Category> = Vec::with_capacity(rows.len());

    for row in rows {
        categories.push(Category { name: row.get(0) })
    }

    let categories_response = CategoriesResponse {
        categories: categories,
    };

    let api_response = APIGatewayResponse::new(200, &categories_response).unwrap();
    Ok(api_response)
}
