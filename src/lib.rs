use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_keyvalue::{
    GetResponse, KeyValue, KeyValueSender, SetAddRequest, SetDelRequest, SetRequest,
};

mod azure;
use azure::*;

const WASMCLOUD_GUNMETAL_COLOR: &str = "253746";
const OFFICIAL_KEY_PREFIX: &str = "wasmcloud:category";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct OcirefferActor {}

#[derive(serde::Deserialize, serde::Serialize)]
struct ReferenceInformation {
    name: String,
    url: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct OfficialPost<'a> {
    category: &'a str,
    name: &'a str,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct OfficialGet<'a> {
    category: &'a str,
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
    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        Ok(match (&req.method[..], &req.path[..]) {
            // Manually POST a reference
            ("POST", "/api/reference") => {
                if let Ok(update) = serde_json::from_slice::<ReferenceInformation>(&req.body) {
                    store_reference(ctx, &update.name, &update.url).await
                } else {
                    HttpResponse::bad_request("Payload did not contain provider name and url")
                }
            }
            // Invoked via Azure webhooks
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
            // Add a name to an "official" wasmCloud list
            ("POST", "/category") => {
                if let Ok(req) = serde_json::from_slice::<OfficialPost>(&req.body) {
                    add_official(ctx, req.category, req.name).await
                } else {
                    HttpResponse::bad_request(
                        "Payload did not contain required category and name fields",
                    )
                }
            }
            // Retrieve the "official" wasmCloud resources
            ("GET", "/category") => {
                if let Ok(req) = serde_json::from_slice::<OfficialGet>(&req.body) {
                    HttpResponse::ok(fetch_official(ctx, req.category).await)
                } else {
                    HttpResponse::bad_request("Payload did not contain required category field")
                }
            }
            ("DELETE", "/category") => {
                if let Ok(req) = serde_json::from_slice::<OfficialPost>(&req.body) {
                    remove_official(ctx, req.category, req.name).await
                } else {
                    HttpResponse::bad_request(
                        "Payload did not contain required category and name fields",
                    )
                }
            }
            ("GET", name) => HttpResponse::ok(
                serde_json::to_vec(&ShieldsResponse::new(
                    "",
                    &fetch_reference(ctx, name.trim_matches('/'))
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

//
// Individual functions
//

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
    KeyValueSender::new()
        .get(ctx, name)
        .await
        .ok()
        .filter(|r| r.exists)
        .map(|r| r.value)
}

//
// "Official" set functions
//

async fn add_official(ctx: &Context, category: &str, name: &str) -> HttpResponse {
    if let Err(e) = KeyValueSender::new()
        .set_add(
            ctx,
            &SetAddRequest {
                set_name: format!("{OFFICIAL_KEY_PREFIX}:{category}"),
                value: name.to_string(),
            },
        )
        .await
    {
        HttpResponse::internal_server_error(format!("Failed to store official {category}: {e:?}",))
    } else {
        HttpResponse::ok(format!("Official {category} {name} added"))
    }
}

async fn remove_official(ctx: &Context, category: &str, name: &str) -> HttpResponse {
    if let Err(e) = KeyValueSender::new()
        .set_del(
            ctx,
            &SetDelRequest {
                set_name: format!("{OFFICIAL_KEY_PREFIX}:{category}"),
                value: name.to_string(),
            },
        )
        .await
    {
        HttpResponse::internal_server_error(format!("Failed to remove official {category}: {e:?}",))
    } else {
        HttpResponse::ok(format!("Official {category} {name} removed"))
    }
}

async fn fetch_official(ctx: &Context, category: &str) -> String {
    let keyvalue = KeyValueSender::new();
    let all_official = keyvalue
        .set_query(ctx, &format!("{OFFICIAL_KEY_PREFIX}:{category}"))
        .await
        .ok()
        .unwrap_or_default();

    let mut references = vec![];
    for official in all_official {
        if let Ok(GetResponse {
            exists: true,
            value,
        }) = keyvalue.get(ctx, &official).await
        {
            references.push(ReferenceInformation {
                name: official.to_owned(),
                url: value.to_owned(),
            });
        }
    }

    serde_json::to_string(&references).unwrap_or_default()
}
