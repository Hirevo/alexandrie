Crate search endpoint
=====================

This endpoint allows to search through all of the crates ever published to the registry.  
This is notably called by the `cargo search` command.  
Being essential to Cargo and the registry, this endpoint is documented in the [**official Cargo book**](https://doc.rust-lang.org/cargo/reference/registries.html#search).  

**Endpoint URL**: `/api/v1/crates`  
**HTTP Method**: `GET`  
**Endpoint Type:** Public  

HTTP Query Parameters
---------------------

This endpoint accepts the following query parameters:

- **(required)** `q`: The query string for the search (like `serde json` to possibly find `serde_json`).
- `page`: The non-zero page number to retrive (defaults to `1`).
- `per_page`: The non-zero number of results per page (default to `15`).

Responses
---------

**Status:** `200 OK`

**Body:**

The registry will return an object of the following shape (as specified in [**The Cargo book**](https://doc.rust-lang.org/cargo/reference/registries.html#search)):

```js
{
    // Array of results.
    "crates": [
        {
            // Name of the crate.
            "name": "rand",
            // The highest version available.
            "max_version": "0.6.1",
            // Textual description of the crate.
            "description": "Random number generators and other randomness functionality.",
        }
    ],
    "meta": {
        // Total number of results available on the server.
        "total": 119
    }
}
```

There may be more fields added to each results but be wary of depending on those.
