use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_keyvalue::{GetResponse, KeyValue, KeyValueSender, SetRequest};

mod azure;
use azure::*;

const WASMCLOUD_GUNMETAL_COLOR: &str = "253746";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct OcirefferActor {}

#[derive(serde::Deserialize, serde::Serialize)]
struct ReferenceInformation<'a> {
    name: &'a str,
    url: &'a str,
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

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for OcirefferActor {
    async fn handle_request(
        &self,
        ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        Ok(match (&req.method[..], &req.path[..]) {
            ("POST", "/api/reference") => {
                if let Ok(update) = serde_json::from_slice::<ReferenceInformation>(&req.body) {
                    store_reference(ctx, update.name, update.url).await
                } else {
                    HttpResponse::bad_request("Payload did not contain provider name and url")
                }
            }
            ("POST", "/api/azurehook") => {
                if let Ok(event) = serde_json::from_slice::<RequestPayload>(&req.body) {
                    let name = &event.target.repository;
                    let url = format!(
                        "{}/{}:{}",
                        event.request.host, event.target.repository, event.target.tag
                    );
                    store_reference(ctx, name, &url).await
                } else {
                    HttpResponse::bad_request(
                        "Azure webhook payload did not contain required fields",
                    )
                }
            }
            ("GET", path) => HttpResponse::ok(
                serde_json::to_vec(&ShieldsResponse::new(
                    "",
                    &fetch_reference(ctx, path.trim_matches('/'))
                        .await
                        .unwrap_or_else(|| "Provider not yet published".to_string()),
                    WASMCLOUD_GUNMETAL_COLOR,
                ))
                .unwrap_or_else(|_| "Failed to serialize response, for some ungodly reason".into()),
            ),
            _ => HttpResponse::not_found(),
        })
    }
}

async fn store_reference(ctx: &Context, name: &str, url: &str) -> HttpResponse {
    if let Err(e) = KeyValueSender::new()
        .set(
            ctx,
            &SetRequest {
                key: name.to_string(),
                value: url.to_string(),
                expires: 0,
            },
        )
        .await
    {
        HttpResponse::internal_server_error(format!("Failed to store url: {e:?}",))
    } else {
        HttpResponse::ok(format!("Url {url} stored for {name}"))
    }
}

async fn fetch_reference(ctx: &Context, name: &str) -> Option<String> {
    if let Ok(GetResponse {
        exists: true,
        value,
    }) = KeyValueSender::new().get(ctx, name).await
    {
        Some(value)
    } else {
        None
    }
}
