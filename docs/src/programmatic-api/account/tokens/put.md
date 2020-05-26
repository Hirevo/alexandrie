Token generation endpoint
=========================

This endpoint allows to generate a new authentication token.  

**Endpoint URL**: `/api/v1/account/tokens`  
**HTTP Method**: `PUT`  
**Endpoint Type:** Authenticated  

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // The name for the new authentication token.
    "name": "Continuous Integration"
}
```

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // The generated authentication token for that account.
    "token": "dfe966790098b9123a098e6a7"
}
```
