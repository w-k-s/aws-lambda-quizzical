use http::StatusCode;
use lambda::{error::HandlerError, Context};
use log::info;
use models::ValidationError;
use repositories::{
    RepositoryError,
    RepositoryError::{ConnectionError, ConversionError, DatabaseError, IOError, UnknownError},
};
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Error as JSONError};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::str::FromStr;

/* #region APIGatewayEvent */

/**
 * API Gateway Event takes advantage of AWS Lambda proxy integration.
 * All parameters are passed in an event structure.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct APIGatewayEvent {
    pub path: String,
    #[serde(rename = "queryStringParameters")]
    pub query: Option<HashMap<String, String>>,
    #[serde(rename = "pathParameters")]
    pub path_parameters: Option<HashMap<String, String>>,
    pub body: Option<String>,
}

impl APIGatewayEvent {
    pub fn parse<'a, T>(&'a self) -> Result<Option<T>, APIErrorResponse>
    where
        T: Deserialize<'a>,
    {
        match self.body {
            Some(ref body) => from_str(body).map_err(|e| {
                APIErrorType::ParsingError {
                    id: None,
                    detail: Some(format!("{}", e)),
                }
                .into()
            }),
            None => Ok(None),
        }
    }

    pub fn parse_with_validator<'a, T>(
        &'a self,
        validator: &Fn(&T) -> Result<(), ValidationError>,
    ) -> Result<Option<T>, APIErrorResponse>
    where
        T: Deserialize<'a>,
    {
        match self.body {
            Some(ref body) => from_str(body)
                .map_err(|e| {
                    APIErrorType::ParsingError {
                        id: None,
                        detail: Some(format!("{}", e)),
                    }
                    .into()
                })
                .and_then(|t: T| match validator(&t) {
                    Ok(_) => Ok(Some(t)),
                    Err(e) => Err(APIErrorType::ValidationError {
                        id: None,
                        detail: Some(format!("{}", e)),
                    }
                    .into()),
                }),
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

    pub fn get_path_param<T>(&self, name: &str) -> Option<T>
    where
        T: FromStr,
    {
        match self.path_parameters {
            Some(ref params) => params.get(name).and_then(|value| T::from_str(value).ok()),
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
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl APIGatewayResponse {
    pub fn new<T: Serialize>(
        status_code: u16,
        data: Option<&T>,
    ) -> Result<APIGatewayResponse, JSONError> {
        let mut headers = HashMap::new();
        headers.insert("Access-Control-Allow-Origin".to_owned(), "*".to_owned());
        let body: String = match data {
            Some(ref data) => to_string(data)?,
            None => "".into(),
        };
        Ok(APIGatewayResponse {
            status_code: status_code,
            headers: headers,
            body: body,
        })
    }

    pub fn parse<'a, T>(&'a self) -> Result<T, JSONError>
    where
        T: Deserialize<'a>,
    {
        from_str(&self.body)
    }
}

impl std::fmt::Display for APIGatewayResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let headers: Vec<String> = self
            .headers
            .iter()
            .map(|e| format!("{}:{}", e.0, e.1))
            .collect();
        write!(
            f,
            "HTTP {}\n{}\n\n{}\n",
            self.status_code,
            headers.join("\n"),
            self.body
        )
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
    handler: &Fn(APIGatewayEvent, Config) -> Result<APIGatewayResponse, APIErrorResponse>,
) -> Result<APIGatewayResponse, HandlerError> {
    info!("APIGatewayEvent: {}", event);

    let config = Config::with_context(&context);

    Ok(match handler(event, config) {
        Ok(response) => response,
        Err(error) => APIGatewayResponse::new(error.status_code(), Some(&error))
            .map_err(|e| context.new_error(&format!("{}", e)))?,
    })
}

/* #APIErrorResponse */

#[derive(Debug, Serialize, Deserialize)]
pub struct APIError {
    id: Option<String>,
    status: u16,
    code: String,
    title: String,
    detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIErrorResponse {
    errors: Vec<APIError>,
}

impl APIErrorResponse {
    pub fn error(
        id: Option<String>,
        status: u16,
        code: String,
        title: String,
        detail: Option<String>,
    ) -> Self {
        return APIErrorResponse {
            errors: vec![APIError {
                id: id,
                status: status,
                code: code,
                title: title,
                detail: detail,
            }],
        };
    }
}

impl APIErrorResponse {
    pub fn status_code(&self) -> u16 {
        self.errors
            .first()
            .and_then(|error| Some(error.status))
            .unwrap_or(500u16)
    }
}

pub enum APIErrorType {
    ParsingError {
        id: Option<String>,
        detail: Option<String>,
    },
    ValidationError {
        id: Option<String>,
        detail: Option<String>,
    },
    RepositoryError {
        repositoryError: RepositoryError,
    },
}

impl std::convert::From<APIErrorType> for APIErrorResponse {
    fn from(pattern: APIErrorType) -> Self {
        return match pattern {
            APIErrorType::ParsingError { id, detail } => APIErrorResponse::error(
                id,
                400,
                "validation.request".into(),
                "Invalid Request Parameters".into(),
                detail,
            ),
            APIErrorType::ValidationError { id, detail } => APIErrorResponse::error(
                id,
                400,
                "validation".into(),
                "Validation Error".into(),
                detail,
            ),
            APIErrorType::RepositoryError { repositoryError } => repositoryError.into(),
        };
    }
}

impl std::convert::From<RepositoryError> for APIErrorResponse {
    fn from(error: RepositoryError) -> Self {
        let (code, title, detail) = match error {
            ConnectionError(message) => ("db.connection", "Database Connection", Some(message)),
            DatabaseError(_, message) => (
                "db.execution",
                "Database Execution",
                Some(format!("psql {}", message)),
            ),
            ConversionError(message) => ("db.data", "Database Data Error", Some(message)),
            IOError(message) => ("db.io", "Database IO Error", Some(message)),
            UnknownError(message) => ("db", "Database Unknown Error", message),
        };

        APIErrorResponse::error(None, 500, code.into(), title.into(), detail)
    }
}
