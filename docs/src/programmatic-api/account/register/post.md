Account registration endpoint
=============================

This endpoint allows to register for a new account in the registry and obtain an authentication token.  

**Endpoint URL**: `/api/v1/account/register`  
**HTTP Method**: `POST`  
**Endpoint Type:** Public  

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // The account's email.
    "email": "john.doe@example.com",
    // The account's displayable name.
    "name": "John Doe",
    // The password for that account.
    "passwd": "my-superb-password"
}
```

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // An authentication token for that account.
    "token": "dfe966790098b9123a098e6a7"
}
```
