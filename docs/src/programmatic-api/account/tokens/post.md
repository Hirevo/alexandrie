Token information endpoint (from token)
=======================================

This endpoint allows to get information about a specific token if issued for the same account.  

This endpoint gives information about a token from the token itself, so the token is sent as a `POST` request to avoid exposing the token in the URL.  

**Endpoint URL**: `/api/v1/account/tokens`  
**HTTP Method**: `POST`  
**Endpoint Type:** Authenticated  

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // The authentication token to get information for.
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
    // The displayable name for the token.
    "name": "Continous Integration"
}
```
