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
use http::StatusCode;
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
) -> Result<APIGatewayResponse, APIErrorResponse> {
    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let categories = CategoriesRepository { conn: conn }.list_categories()?;
    let api_response = APIGatewayResponse::new(StatusCode::OK, Some(&categories)).unwrap();

    Ok(api_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{Categories, Category};
    use std::time::SystemTime;

    #[test]
    fn test_categories_returns_200_with_list() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        let event = APIGatewayEvent {
            path: "/".into(),
            query: None,
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        let title = format!("{:?}", SystemTime::now());
        let conn = Arc::new(connect_db_with_conn_string(&config.connection_string).unwrap());
        let _ = conn.execute(
            "INSERT INTO categories (name,active) VALUES($1,$2)",
            &[&title, &true],
        );

        match categories_handler(event, config) {
            Err(_) => assert!(false),
            Ok(resp) => {
                assert_eq!(resp.status_code(), StatusCode::OK);

                let categories: Categories = resp.parse().unwrap();
                assert!(categories.categories.len() >= 1);

                assert_eq!(
                    1,
                    categories
                        .categories
                        .iter()
                        .filter(|c| c.title == title)
                        .count()
                );
            }
        }
    }
}
