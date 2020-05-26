Token revocation endpoint
=========================

This endpoint allows to revoke an existing authentication token issued for the same account.  

**Endpoint URL**: `/api/v1/account/tokens`  
**HTTP Method**: `DELETE`  
**Endpoint Type:** Authenticated  

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // The authentication token to revoke.
    "token": "dfe966790098b9123a098e6a7"
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
    "revoked": true
}
```
