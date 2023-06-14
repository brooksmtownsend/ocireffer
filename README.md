# ocireffer Actor

This actor is used to store and retrive the official OCI references for wasmCloud capability providers. Though it's used for providers primarily, there's no reason why it couldn't be used for actors as well.

There are two endpoints this actor supports:

| Endpoint        | Method | Description                                | Payload                                                                     |
| --------------- | ------ | ------------------------------------------ | --------------------------------------------------------------------------- |
| `/api/provider` | `POST` | Stores a new OCI reference for a provider  | `{ "name": "httpserver", "url": "wasmcloud.azurecr.io/httpserver:0.17.0" }` |
| `/<provider>`   | `GET`  | Retrieves the OCI reference for a provider | N/A                                                                         |

## Example Usage

Assuming this actor is running on `localhost:8080`, you can store a new provider reference with:

```shell
curl -X POST -H "Content-Type: application/json" -d '{"name": "httpserver", "url": "wasmcloud.azurecr.io/httpserver:0.17.0"}' http://localhost:8080/api/provider
```

And you can retrieve a provider reference with:

```shell
curl http://localhost:8080/httpserver
```
