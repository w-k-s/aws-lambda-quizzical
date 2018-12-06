extern crate lambda_runtime as lambda;
extern crate serde_derive;
extern crate log;
extern crate simple_logger;

use serde_derive::{Serialize, Deserialize};
use lambda::{lambda, Context, error::HandlerError};
use log::error;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct GreetingEvent {
    greeting: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct GreetingResponse {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    lambda!(my_handler);

    Ok(())
}

fn my_handler(event: GreetingEvent, ctx: Context) -> Result<GreetingResponse, HandlerError> {
    if event.name == "" {
        error!("Empty name in request {}", ctx.aws_request_id);
        return Err(ctx.new_error("Empty name"));
    }

    Ok(GreetingResponse {
        message: format!("{}, {}!", event.greeting, event.name),
    })
}