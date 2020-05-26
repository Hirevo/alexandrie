Crate publication endpoint
==========================

This endpoint allows to publish a new crate version to the registry.  
This is notably called by the `cargo publish` command.  
Being essential to Cargo and the registry, this endpoint is documented in the [**official Cargo book**](https://doc.rust-lang.org/cargo/reference/registries.html#publish).  

**Endpoint URL**: `/api/v1/crates/new`  
**HTTP Method**: `PUT`  
**Endpoint Type:** Authenticated  

HTTP Request Body
-----------------

The format of the request body expected by this endpoint is a bit special and is not easily constructed manually by humans.  
It is also described in [**The Cargo book**](https://doc.rust-lang.org/cargo/reference/registries.html#publish).

The body is composed of:

- A 32-bit unsigned little-endian integer of the length of the following JSON data.
- A JSON object detailing metadata about the package.
- A 32-bit unsigned little-endian integer of the length of the crate archive.
- The crate archive itself (as a `.tar.gz` binary file).

Responses
---------

**Status:** `200 OK`

**Body:**  
Currently, the endpoint returns an empty json object as the response body.  
But The Cargo book allows adding an optional `warnings` object, like so:

```js
{
    // Optional object of warnings to display to the user.
    "warnings": {
        // Array of strings of categories that are invalid and ignored.
        "invalid_categories": [],
        // Array of strings of badge names that are invalid and ignored.
        "invalid_badges": [],
        // Array of strings of arbitrary warnings to display to the user.
        "other": []
    }
}
```

So keep in mind that the registry may make use of this object at any time.
