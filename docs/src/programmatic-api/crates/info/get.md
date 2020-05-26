Crate information endpoint
==========================

This endpoint allows to get detailed information about a specific crate of the registry.  

**Endpoint URL**: `/api/v1/crates/<name>`  
**HTTP Method**: `GET`  
**Endpoint Type:** Public  

HTTP Path Parameters
--------------------

This endpoint accepts the following path parameters (as shown in the endpoint's URL):

- **(required)** `name`: The name of the crate (like `serde_json`).

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // Name of the crate.
    "name": "rand",
    // The highest version available.
    "max_version": "0.6.1",
    // Textual description of the crate.
    "description": "Random number generators and other randomness functionality.",
    // Optional link to the development repository of the crate.
    "repository": "https://github.com/rust-random/rand",
    // Optional link to the documentation of the crate.
    "documentation": "https://docs.rs/rand",
    // The crate's download count.
    "downloads": 34464729,
    // The crate's creation date (in the 'YY-MM-DD hh:mm:ss' format).
    "created_at": "2015-02-03 06:17:14",
    // The crate's last modification date (in the 'YY-MM-DD hh:mm:ss' format).
    "updated_at": "2020-01-10 21:46:21",
    // The crate's keywords.
    "keywords": [
        "rng",
        "random"
    ],
    // The crate's categories.
    "categories": [
        "no-std",
        "algorithms"
    ],
}
```
