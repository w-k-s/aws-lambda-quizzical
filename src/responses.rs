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
    pub fn new(
        data: Vec<T>,
        page: u32,
        total_records: u32,
        mut limit: u32,
    ) -> PaginatedResponse<T> {
        if limit <= 0 {
            limit = 1;
        }

        let page_count =  (total_records as f64/ limit as f64).ceil() as u32;

        let size = data.len() as u32;
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_total_records_less_than_limit() {
        let names : Vec<String> = vec!["james".into(), "bond".into(), "harry".into(), "potter".into(), "ron".into()];
        let response = PaginatedResponse::new(names, 1u32, 5u32, 10u32);
    
        assert_eq!(response.page, 1);
        assert_eq!(response.size, 5);
        assert_eq!(response.page_count, 1);
        assert_eq!(response.last, true);
    }

    #[test]
    fn test_total_records_equals_limit() {
        let names : Vec<String> = vec!["james".into(), "bond".into(), "harry".into(), "potter".into(), "ron".into()];
        let response = PaginatedResponse::new(names, 1u32, 5u32, 5u32);
    
        assert_eq!(response.page, 1);
        assert_eq!(response.size, 5);
        assert_eq!(response.page_count, 1);
        assert_eq!(response.last, true);
    }

    #[test]
    fn test_total_records_more_than_limit() {
        let names : Vec<String> = vec!["james".into(), "bond".into(), "harry".into(), "potter".into(), "ron".into()];
        let response = PaginatedResponse::new(names, 1u32, 25u32, 10u32);
    
        assert_eq!(response.page, 1);
        assert_eq!(response.size, 5);
        assert_eq!(response.page_count, 3);
        assert_eq!(response.last, false);
    }

    #[test]
    fn test_last_page_is_true_when_page_is_last() {
        let names : Vec<String> = vec!["james".into(), "bond".into(), "harry".into(), "potter".into(), "ron".into()];
        let response = PaginatedResponse::new(names, 3u32, 25u32, 10u32);
    
        assert_eq!(response.page, 3);
        assert_eq!(response.size, 5);
        assert_eq!(response.page_count, 3);
        assert_eq!(response.last, true);
    }

    #[test]
    fn test_last_page_is_true_when_page_is_last2() {
        let names : Vec<String> = vec!["james".into(), "bond".into(), "harry".into(), "potter".into(), "ron".into()];
        let response = PaginatedResponse::new(names, 4u32, 25u32, 10u32);
    
        assert_eq!(response.page, 4);
        assert_eq!(response.size, 5);
        assert_eq!(response.page_count, 3);
        assert_eq!(response.last, true);
    }
}
