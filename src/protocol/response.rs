use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Response {
    Ok(Payload),
    Err { message: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Payload {
    Set,
    Get(GetPayload),
    Delete(DeletePayload),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GetPayload {
    Found(Vec<u8>),
    NotFound,
}

#[derive(Debug, Serialize)]
pub enum DeletePayload {
    Removed,
    NotFound,
}

impl From<Payload> for Response {
    fn from(value: Payload) -> Self {
        Response::Ok(value)
    }
}

impl From<GetPayload> for Response {
    fn from(value: GetPayload) -> Self {
        Response::Ok(Payload::Get(value))
    }
}

impl From<DeletePayload> for Response {
    fn from(value: DeletePayload) -> Self {
        Response::Ok(Payload::Delete(value))
    }
}

pub fn serialize_response(response: Result<Response>) -> Result<Vec<u8>> {
    let response = match response {
        Ok(r) => r,
        Err(e) => Response::Err {
            message: e.to_string(),
        },
    };

    let mut bytes = serde_json::to_vec(&response)?;
    bytes.push(b'\n');
    Ok(bytes)
}
