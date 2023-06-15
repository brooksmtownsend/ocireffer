# ocireffer Actor

This actor is used to store and retrive the official OCI references for wasmCloud resources in AzureCR. There are a few endpoints this actor supports:

| Endpoint         | Method | Description                                                                         | Payload                                                                                                                                                                            |
| ---------------- | ------ | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/api/reference` | `POST` | Stores a new OCI reference                                                          | `{ "name": "httpserver", "url": "wasmcloud.azurecr.io/httpserver:0.17.0" }`                                                                                                        |
| `/api/azurehook` | `POST` | Stores a new OCI reference, from a webhook                                          | Refer to [the Azure docs](https://learn.microsoft.com/en-us/azure/container-registry/container-registry-webhook-reference#payload-example-image-push-event) for the payload format |
| `/<provider>`    | `GET`  | Retrieves the OCI reference for a stored entity                                     | N/A                                                                                                                                                                                |
| `/category`      | `POST` | Adds an item to a category, name should match the name of the reference stored      | `{ "category": "providers", "name": "httpserver" }`                                                                                                                                |
| `/category`      | `GET`  | Retrives all items in a category                                                    | `{ "category": "providers" }`                                                                                                                                                      |
| `/category`      | `DEL`  | Deletes an item from a category, name should match the name of the reference stored | `{ "category": "providers", "name": "httpserver" }`                                                                                                                                |

## Example Usage

Assuming this actor is running on `localhost:8080`, you can store a new reference with:

```shell
curl -X POST -H "Content-Type: application/json" -d '{"name": "httpserver", "url": "wasmcloud.azurecr.io/httpserver:0.17.0"}' http://localhost:8080/api/reference
```

And you can retrieve a provider reference with:

```shell
curl http://localhost:8080/httpserver
```
