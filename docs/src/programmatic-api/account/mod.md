Account Management endpoints
============================

In this section, you will find documentation about all the account management endpoints of the programmatic API.  

Public Endpoints
----------------

Public endpoints are accessible without needing to authenticate.

- [**Login**](login/post.md): **`POST /api/v1/account/login`**
- [**Register**](register/post.md): **`POST /api/v1/account/register`**

Authenticated Endpoints
-----------------------

The following endpoints require a valid token to be specified in the `Authorization` request header.  
Refer to the [**Authentication docs**](../authentication.md) to learn how to get one and how to use it.  

- [**Get token information (from name)**](tokens/get.md): **`GET /api/v1/account/tokens/<name>`**
- [**Get token information (from token)**](tokens/post.md): **`POST /api/v1/account/tokens`**
- [**Generate authentication token**](tokens/post.md): **`POST /api/v1/account/tokens`**
- [**Revoke authentication token**](tokens/delete.md): **`DELETE /api/v1/account/tokens`**
