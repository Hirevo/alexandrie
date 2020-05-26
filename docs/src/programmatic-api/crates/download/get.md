Crate archive download endpoint
===============================

This endpoint allows to download the archive of a specific crate version from the registry (as a `.tar.gz` file).  
This is notably called by the `cargo build` and `cargo fetch` commands, when downloading dependencies.  

**Endpoint URL**: `/api/v1/crates/<name>/<version>/download`  
**HTTP Method**: `GET`  
**Endpoint Type:** Public  

HTTP Path Parameters
--------------------

This endpoint accepts the following path parameters (as shown in the endpoint's URL):

- **(required)** `name`: The name of the crate (like `serde_json`).
- **(required)** `version`: The version of the crate (like `3.1.23`).

Responses
---------

**Status:** `200 OK`

**Body:**

The registry will send back the crate archive as binary data with an `application/octet-stream` content-type header.  
The binary data is the content of the `.tar.gz` archive stored for this specific version of the crate.  
