Crate owners addition endpoint
==============================

This endpoint allows to grant owner privileges to some authors for a given crate.  

You need to be an owner of that crate in order to use this endpoint.  

**Endpoint URL**: `/api/v1/crates/<name>/owners`  
**HTTP Method**: `PUT`  
**Endpoint Type:** Authenticated  

HTTP Path Parameters
--------------------

This endpoint accepts the following path parameters (as shown in the endpoint's URL):

- **(required)** `name`: The name of the crate (like `serde_json`).

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // Array of user emails
    "users": [
        "john.doe@example.com",
        "nicolas@polomack.eu"
    ]
}
```

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // Whether the operation went well.
    "ok": "true",
    // A human-displayable message describing the operation's outcome.
    "msg": "John Doe and Nicolas Polomack has been added as authors",
}
```
