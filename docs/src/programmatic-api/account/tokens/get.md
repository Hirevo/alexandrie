Token information endpoint (from name)
======================================

This endpoint allows to get information about a specific token if issued for the same account.  

**Endpoint URL**: `/api/v1/account/tokens/<name>`  
**HTTP Method**: `GET`  
**Endpoint Type:** Authenticated  

HTTP Path Parameters
--------------------

This endpoint accepts the following path parameters (as shown in the endpoint's URL):

- **(required)** `name`: The name of the token.

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
