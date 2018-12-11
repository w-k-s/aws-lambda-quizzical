extern crate postgres;

use apigateway::{APIError, APIErrorResponse};
use http::StatusCode;
use postgres::{Connection, TlsMode};
use std::env;
use std::fmt;

pub enum ConnectionError {
    ConnectionStringNotFound(String),
    ConnectionFailed(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionError::ConnectionStringNotFound(ref conn_string) => write!(
                f,
                "The env variable '{}' did not contain connection string",
                conn_string
            ),
            ConnectionError::ConnectionFailed(ref description) => write!(f, "{}", description),
        }
    }
}

impl std::convert::From<ConnectionError> for APIError {
    fn from(error: ConnectionError) -> Self {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            APIErrorResponse {
                message: format!("{}", error),
            },
        )
    }
}

pub fn connect_db_using_env_var(conn_string_env_var: &str) -> Result<Connection, ConnectionError> {
    let conn_string_var = env::var_os(conn_string_env_var)
        .map(|host| host.into_string().expect("invalid CONN_STRING"));

    let conn_string = match conn_string_var {
        Some(var) => var,
        None => {
            return Err(ConnectionError::ConnectionStringNotFound(
                conn_string_env_var.to_string(),
            ))
        }
    };

    Ok(Connection::connect(conn_string, TlsMode::None)
        .map_err(|e| ConnectionError::ConnectionFailed(format!("{}", e)))?)
}
