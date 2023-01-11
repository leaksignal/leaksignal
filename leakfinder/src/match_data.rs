use std::collections::HashMap;

#[derive(Debug)]
pub struct Match {
    pub category_name: String,
    pub global_start_position: Option<u64>,
    pub global_length: Option<u64>,
    pub matcher_path: String,
    pub matched_value: Option<String>,
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct ParsedHeaderMatches {
    pub name: String,
    pub matches: ParsedMatches,
}

#[derive(Debug)]
pub struct EvaluationOutput {
    pub policy_id: String,
    pub time_request_start: u64,
    pub time_response_start: u64,
    pub policy_path: String,
    pub token: String,
    pub ip: String,
    pub request_headers: Vec<ParsedHeaderMatches>,
    pub response_headers: Vec<ParsedHeaderMatches>,
    pub response: ParsedMatches,
    pub request: ParsedMatches,
}

#[derive(Debug)]
pub struct ParsedMatches {
    pub matches: Vec<Match>,
    pub body_size: u64,
    pub body: Option<Vec<u8>>,
    pub category_performance_us: HashMap<String, u64>,
    pub time_parse_start: u64,
    pub time_parse_end: u64,
}
