use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Error as JSONError};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
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
