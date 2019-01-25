extern crate postgres;

use apigateway::APIErrorResponse;
use postgres::{Connection, TlsMode};
use repositories::RepositoryError;

pub fn connect_db_with_conn_string(conn_string: &str) -> Result<Connection, APIErrorResponse> {
    Connection::connect(conn_string, TlsMode::None).map_err(|e| RepositoryError::from(e).into())
}
