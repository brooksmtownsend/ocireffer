use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct OcirefferActor {}

const ACR_PREFIX: &str = "wasmcloud.azurecr.io";
const WASMCLOUD_GUNMETAL_COLOR: &str = "253746";

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for OcirefferActor {
    async fn handle_request(
        &self,
        _ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        let provider_name = req.path.trim_matches('/');
        Ok(HttpResponse {
            body: serde_json::to_vec(&ShieldsResponse::new(
                "",
                &oci_url(provider_name),
                WASMCLOUD_GUNMETAL_COLOR,
            ))
            .unwrap_or_default(),
            ..Default::default()
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ShieldsResponse {
    #[serde(rename = "schemaVersion")]
    schema_version: u8,
    label: String,
    message: String,
    color: String,
    #[serde(rename = "namedLogo")]
    named_logo: String,
}

impl ShieldsResponse {
    fn new(label: &str, message: &str, color: &str) -> Self {
        ShieldsResponse {
            schema_version: 1,
            label: label.to_string(),
            message: message.to_string(),
            color: color.to_string(),
            named_logo: "wasmcloud".to_string(),
        }
    }
}

fn oci_url(provider_name: &str) -> String {
    match provider_version(provider_name) {
        None => "Provider not yet published".to_string(),
        Some(ver) => format!("{}/{}:{}", ACR_PREFIX, provider_name, ver),
    }
}

fn provider_version(provider_name: &str) -> Option<&str> {
    match provider_name {
        "blobstore-fs" => Some("0.2.0"),
        "blobstore-s3" => Some("0.3.0"),
        "httpclient" => Some("0.6.0"),
        "httpserver" => Some("0.17.0"),
        "kv-vault" => Some("0.3.0"),
        "kvredis" => Some("0.19.0"),
        "lattice-controller" => Some("0.10.0"),
        "nats_messaging" => Some("0.15.0"),
        "sqldb-postgres" => Some("0.4.0"),
        _ => None,
    }
}
