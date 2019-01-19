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

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct CategoryStatus {
    active: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    start(
        |event: APIGatewayEvent, c: Context| {
            lambda_adapter(event, c, &update_category_active_handler)
        },
        None,
    );
    Ok(())
}

fn update_category_active_handler(
    event: APIGatewayEvent,
    config: Config,
) -> Result<APIGatewayResponse, APIErrorResponse> {
    let category: String = match event.get_query("category") {
        Some(category) => category,
        None => {
            return Err(APIErrorResponse::new(
                StatusCode::BAD_REQUEST,
                "category required in query".into(),
            ))
        }
    };
    let active: bool = match event.get_query("active") {
        Some(active) => active,
        None => {
            return Err(APIErrorResponse::new(
                StatusCode::BAD_REQUEST,
                "active boolean ('true'/'false') required in query".into(),
            ))
        }
    };

    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let active = CategoriesRepository { conn: conn }.set_category_active(&category, active)?;
    let api_response =
        APIGatewayResponse::new(StatusCode::OK, Some(&CategoryStatus { active: active })).unwrap();

    Ok(api_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::Category;
    use std::collections::HashMap;
    use std::time::SystemTime;

    #[test]
    fn test_update_category_active_returns_204_with_status_active() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
        let title = format!("{:?}", SystemTime::now());

        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("category".into(), title.clone());
        query.insert("active".into(), "true".into());

        let event = APIGatewayEvent {
            path: "/category/Science/activate".into(),
            query: Some(query),
            body: None,
        };

        let config = Config {
            connection_string: std::env::var("TEST_CONN_STRING").unwrap(),
        };

        let conn = Arc::new(connect_db_with_conn_string(&config.connection_string).unwrap());
        let _ = conn.execute(
            "INSERT INTO categories (name,active) VALUES($1,$2)",
            &[&title, &false],
        );

        match update_category_active_handler(event, config) {
            Err(e) => {
                print!("{:?}", e);
                assert!(false)
            }
            Ok(resp) => {
                assert_eq!(resp.status_code(), 200);

                let category_status = resp.parse::<CategoryStatus>().unwrap();
                assert!(category_status.active);

                let count_rows = &conn
                    .query(
                        "SELECT COUNT(active) FROM categories WHERE name = $1 AND active=true",
                        &[&title],
                    )
                    .unwrap();

                let count: i64 = match count_rows.is_empty() {
                    true => 0,
                    false => count_rows.get(0).get(0),
                };
                assert_eq!(count, 1);
            }
        }
    }
}
