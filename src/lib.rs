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
struct ProviderUpdate<'a> {
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
        match (&req.method[..], &req.path[..]) {
            ("POST", "/api/provider") => {
                if let Ok(update) = serde_json::from_slice::<ProviderUpdate>(&req.body) {
                    store_provider(ctx, update.name, update.url).await
                } else {
                    Ok(HttpResponse::bad_request(
                        "Payload did not contain provider name and url",
                    ))
                }
            }
            ("POST", "/api/azurehook") => {
                if let Ok(event) = serde_json::from_slice::<RequestPayload>(&req.body) {
                    let name = &event.target.repository;
                    let url = format!(
                        "{}/{}:{}",
                        event.request.host, event.target.repository, event.target.tag
                    );
                    store_provider(ctx, name, &url).await
                } else {
                    Ok(HttpResponse::bad_request(
                        "Azure webhook payload did not contain required fields",
                    ))
                }
            }
            ("GET", path) => {
                let provider_name = path.trim_matches('/');
                Ok(HttpResponse::ok(
                    serde_json::to_vec(&ShieldsResponse::new(
                        "",
                        &provider_url(ctx, provider_name)
                            .await
                            .unwrap_or_else(|| "Provider not yet published".to_string()),
                        WASMCLOUD_GUNMETAL_COLOR,
                    ))
                    .unwrap_or_default(),
                ))
            }
            _ => Ok(HttpResponse::not_found()),
        }
    }
}

async fn store_provider(ctx: &Context, name: &str, url: &str) -> RpcResult<HttpResponse> {
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
        Ok(HttpResponse::internal_server_error(format!(
            "Failed to store provider url: {e:?}",
        )))
    } else {
        Ok(HttpResponse::ok(format!(
            "Provider url {url} stored for {name}"
        )))
    }
}

async fn provider_url(ctx: &Context, provider_name: &str) -> Option<String> {
    if let Ok(GetResponse {
        exists: true,
        value,
    }) = KeyValueSender::new().get(ctx, provider_name).await
    {
        Some(value.to_owned())
    } else {
        None
    }
}
