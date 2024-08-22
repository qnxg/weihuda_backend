use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostQueryResultReq {
    pub name: String,
    pub results: Vec<QueryResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: u32,
    pub question: String,
    pub answer: String,
}
