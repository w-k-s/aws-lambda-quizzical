extern crate lambda_runtime as lambda;
extern crate serde_derive;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

use serde_derive::{Serialize, Deserialize};
use serde::{Serialize, Deserialize};
use lambda::{lambda, Context, error::HandlerError};
use std::error::Error;
use std::collections::HashMap;
use serde_json::{Error as JSONError, to_string, from_str};

#[derive(Serialize, Deserialize)]
struct GreetingRequest{
    greeting: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GreetingResponse{
    message: String,
}

#[derive(Serialize, Deserialize)]
struct APIGatewayEvent {
    path: String,
    body: String,
}

impl APIGatewayEvent{
    fn parse<'a, T>(&'a self)->Result<T, JSONError> where T: Deserialize<'a>{
        from_str(&self.body)   
    }
}

#[derive(Serialize, Deserialize)]
struct APIGatewayResponse{
    #[serde(rename = "statusCode")] 
    status_code: u32,
    headers: HashMap<String, String>,
    body: String
}

impl APIGatewayResponse{
    fn new<T: Serialize>(status: u32, data: &T)->Result<APIGatewayResponse, JSONError>{
        let mut headers = HashMap::new();
        headers.insert("Access-Control-Allow-Origin".to_owned(), "*".to_owned());
        let body = to_string(data)?;
        Ok(APIGatewayResponse{
            status_code: status,
            headers: headers,
            body: body,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);
    Ok(())
}

fn my_handler(event: APIGatewayEvent, _ctx: Context) -> Result<APIGatewayResponse, HandlerError> {
    let request = event.parse::<GreetingRequest>().unwrap();
    let message = format!("{}, {}!",request.greeting, request.name);
    let greeting = GreetingResponse{message: message};
    let response = APIGatewayResponse::new(200,&greeting).unwrap();
    Ok(response)
}