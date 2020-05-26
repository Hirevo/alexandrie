Account login endpoint
======================

This endpoint allows to log in to an account and obtain an authentication token.  

**Endpoint URL**: `/api/v1/account/login`  
**HTTP Method**: `POST`  
**Endpoint Type:** Public  

HTTP Request Body
-----------------

The request body must be a JSON object of the following shape:

```js
{
    // The account's email.
    "email": "john.doe@example.com",
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
