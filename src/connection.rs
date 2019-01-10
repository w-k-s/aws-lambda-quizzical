extern crate postgres;

use apigateway::{APIError, APIErrorResponse};
use http::StatusCode;
use postgres::{Connection, TlsMode};
use std::fmt;

#[derive(Debug)]
pub enum ConnectionError {
    ConnectionFailed(String),
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
                fields: None,
            },
        )
    }
}

pub fn connect_db_with_conn_string(conn_string: &str) -> Result<Connection, ConnectionError> {
    Ok(Connection::connect(conn_string, TlsMode::None)
        .map_err(|e| ConnectionError::ConnectionFailed(format!("{}", e)))?)
}
