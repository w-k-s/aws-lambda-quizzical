use http::StatusCode;
use lambda::{error::HandlerError, Context};
use log::info;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Error as JSONError};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::str::FromStr;

/* #region APIGatewayEvent */

#[derive(Debug, Serialize, Deserialize)]
pub struct APIGatewayEvent {
    pub path: String,
    #[serde(rename = "queryStringParameters")]
    pub query: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

impl APIGatewayEvent {
    pub fn parse<'a, T>(&'a self) -> Result<Option<T>, JSONError>
    where
        T: Deserialize<'a>,
    {
        match self.body {
            Some(ref body) => from_str(body),
            None => Ok(None),
        }
    }

    pub fn get_query<T>(&self, name: &str) -> Option<T>
    where
        T: FromStr,
    {
        match self.query {
            Some(ref query) => query.get(name).and_then(|value| T::from_str(value).ok()),
            None => None,
        }
    }
}

impl std::fmt::Display for APIGatewayEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "APIGatewayEvent{{path: '{}', query: '{:?}',body: '{:?}'}}",
            self.path, self.query, self.body
        )
    }
}

/* #region APIGatewayResponse */

#[derive(Serialize, Deserialize)]
pub struct APIGatewayResponse {
    #[serde(rename = "statusCode")]
    pub status_code: u32,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl APIGatewayResponse {
    pub fn new<T: Serialize>(status: u32, data: &T) -> Result<APIGatewayResponse, JSONError> {
        let mut headers = HashMap::new();
        headers.insert("Access-Control-Allow-Origin".to_owned(), "*".to_owned());
        let body = to_string(data)?;
        Ok(APIGatewayResponse {
            status_code: status,
            headers: headers,
            body: body,
        })
    }
}

/* #region Configuration */

/**
 * Stores environmental variables and the context in which the lambda is running.Config
 *
 * Typically, a lambda function is passed an event and the lambda context.
 * An instance of the lambda context can not be created, which makes it difficult to write unit tests.
 * The `Config` struct is meant to act as a replacement for `Context` so that unit tests can be written.
 *
 * The second benefit is that it makes it easier to provide environmental variables to the unit tests.
 * In other words, using std::env::set_var() in unit tests is avodided.
 */
pub struct Config {
    pub connection_string: String,
}

impl Config {
    fn with_context(_context: &Context) -> Config {
        let conn_string = env::var("CONN_STRING").expect("CONN_STRING required");

        Config {
            connection_string: conn_string,
        }
    }
}

/* #region Generic Lambda Handler */

pub fn lambda_adapter(
    event: APIGatewayEvent,
    context: Context,
    handler: &Fn(APIGatewayEvent, Config) -> Result<APIGatewayResponse, APIError>,
) -> Result<APIGatewayResponse, HandlerError> {
    info!("APIGatewayEvent: {}", event);

    let config = Config::with_context(&context);

    Ok(match handler(event, config) {
        Ok(response) => response,
        Err(error) => APIGatewayResponse::new(error.0.as_u16() as u32, &error.1)
            .map_err(|e| context.new_error(&format!("{}", e)))?,
    })
}

/* #APIError */

pub type APIError = (StatusCode, APIErrorResponse);

#[derive(Serialize, Deserialize)]
pub struct APIErrorResponse {
    pub message: String,
}

impl std::fmt::Display for APIErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "APIErrorResponse{{error: '{}'}}", self.message)
    }
}
