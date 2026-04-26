use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Command {
    Set { key: String, value: Vec<u8> },
    Get { key: String },
    Delete { key: String },
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: Command,
}

pub fn deserialize_request(line: &str) -> Result<Request> {
    let request: Request = serde_json::from_str(line)?;
    Ok(request)
}
