use anyhow::Result;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Command {
    Set { key: String, value: Vec<u8> },
    Get { key: String },
    Delete { key: String },
}

#[derive(Debug)]
pub enum ResponseKind {
    Set,
    Get(GetResponse),
    Delete(DeleteResponse),
}

#[derive(Debug)]
pub enum GetResponse {
    Found(Vec<u8>),
    NotFound,
}

#[derive(Debug)]
pub enum DeleteResponse {
    Removed,
    NotFound,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: Command,
}

#[derive(Debug)]
pub struct Response {
    pub result: Result<ResponseKind, String>,
}

pub fn deserialize_request(line: &str) -> Result<Request> {
    let request: Request = serde_json::from_str(line)?;
    Ok(request)
}

pub fn serialize_response(request: Request) -> Result<Response> {
    todo!()
}
