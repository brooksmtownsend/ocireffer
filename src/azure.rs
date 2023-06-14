//! Azure Container Registry Webhook Payload
//! All non-critical fields are optional.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestPayload {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub action: String,
    pub target: Target,
    pub request: Request,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    #[serde(rename = "mediaType")]
    #[serde(default)]
    pub media_type: String,
    #[serde(default)]
    pub size: i32,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub length: i32,
    pub repository: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub id: String,
    pub host: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub useragent: String,
}
