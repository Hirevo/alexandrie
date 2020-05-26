Crate owners removal endpoint
=============================

This endpoint allows to declare a specific crate version as no longer being yanked.  

You need to be an owner of that crate in order to use this endpoint.  

**Endpoint URL**: `/api/v1/crates/<name>/unyank`  
**HTTP Method**: `PUT`  
**Endpoint Type:** Authenticated  

HTTP Path Parameters
--------------------

This endpoint accepts the following path parameters (as shown in the endpoint's URL):

- **(required)** `name`: The name of the crate (like `serde_json`).
- **(required)** `version`: The version of the crate (like `3.1.23`).

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // Whether the operation went well.
    "ok": "true",
}
```
