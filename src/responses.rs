extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub page_count: u32,
    pub last: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u32, total_records: u32, limit: u32) -> PaginatedResponse<T> {
        let size = data.len() as u32;
        let page_count = (total_records / limit) + 1u32;
        let last = page >= page_count;
        return PaginatedResponse {
            data: data,
            page: page,
            size: size,
            page_count: page_count,
            last: last,
        };
    }
}
