Crate-related endpoints
=======================

In this section, you will find documentation about all the crate-related endpoints of the programmatic API.  

Public Endpoints
----------------

Public endpoints are accessible without needing to authenticate.

- [**Search crates**](search/get.md): **`POST /api/v1/crates?<q>[&<page>][&<per_page>]`**
- [**Get crate information**](info/get.md): **`GET /api/v1/crates/<name>`**
- [**List crate owners**](owners/put.md): **`GET /api/v1/crates/<name>/owners`**
- [**Download crate archive**](download/get.md): **`GET /api/v1/crates/<name>/<version>/download`**
- [**List crate categories**](categories/get.md): **`GET /api/v1/categories`**

Authenticated Endpoints
-----------------------

The following endpoints require a valid token to be specified in the `Authorization` request header.  
Refer to the [**Authentication docs**](../authentication.md) to learn how to get one and how to use it.  

- [**Publish crate**](publish/put.md): **`PUT /api/v1/crates/new`**
- [**Add crate owners**](owners/put.md): **`PUT /api/v1/crates/<name>/owners`**
- [**Remove crate owners**](owners/delete.md): **`DELETE /api/v1/crates/<name>/owners`**
- [**Yank crate version**](yank/delete.md): **`DELETE /api/v1/crates/<name>/<version>/yank`**
- [**Unyanking crate version**](unyank/put.md): **`PUT /api/v1/crates/<name>/<version>/unyank`**
