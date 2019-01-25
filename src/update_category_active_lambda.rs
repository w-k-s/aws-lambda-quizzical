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

use apigateway::{APIErrorType::*, *};
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
    let category: String = match event.get_path_param("category") {
        Some(category) => category,
        None => {
            return Err(ValidationError {
                id: None,
                detail: Some("category required in path".into()),
            }
            .into())
        }
    };
    let status = match event.parse::<CategoryStatus>() {
        Ok(Some(status)) => status,
        _ => {
            return Err(ValidationError {
                id: None,
                detail: Some("Expected {\"active\": [true|false] }".into()),
            }
            .into())
        }
    };

    let conn = Arc::new(connect_db_with_conn_string(&config.connection_string)?);

    let active =
        CategoriesRepository { conn: conn }.set_category_active(&category, status.active)?;
    let api_response =
        APIGatewayResponse::new(200, Some(&CategoryStatus { active: active })).unwrap();

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

        let mut path_params: HashMap<String, String> = HashMap::new();
        path_params.insert("category".into(), title.clone());
        path_params.insert("active".into(), "true".into());

        let event = APIGatewayEvent {
            path: "/category/Science/activate".into(),
            query: None,
            path_parameters: Some(path_params),
            body: Some("{\"active\": true }".into()),
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
                assert_eq!(resp.status_code, 200);

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
