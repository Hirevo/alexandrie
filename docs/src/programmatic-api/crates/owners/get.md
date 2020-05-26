Crate owners listing endpoint
=============================

This endpoint allows to know who are the owners of a given crate.  

**Endpoint URL**: `/api/v1/crates/<name>/owners`  
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
    // Array of crate owners.
    "users": [
        {
            // The user ID of the owner.
            "id": 63,
            // The login email of the owner.
            "login": "john.doe@example.com",
            // The name of the owner.
            "name": "John Doe"
        }
    ]
}
```
