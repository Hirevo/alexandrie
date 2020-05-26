The Programmatic API
====================

This section covers the programmatic API that Alexandrie exposes.  
Some of these endpoints are used by `cargo` itself when running commands like `cargo search`, `cargo publish` and some others.  

URL format
----------

Before anything, here is a description of the syntax used throughout these pages for writing URLs:

- **`<name>`**: is either:
  - a placeholder for any path segment (when in within the pathname, eg. `/crates/<name>`)
  - a querystring variable (when in querystring position, eg. `/search?<q>`)
- **`[...]`**: means that the the pattern inside these brackets is optional:
  - mostly used for optional querystring variables (eg. `/search?<q>[&<page>]`)

Public Endpoints
----------------

Public endpoints are accessible without needing to authenticate.

**Crates section:**

- [**Search crates**](crates/search/get.md): **`POST /api/v1/crates?<q>[&<page>][&<per_page>]`**
- [**Get crate information**](crates/info/get.md): **`GET /api/v1/crates/<name>`**
- [**List crate owners**](crates/owners/put.md): **`GET /api/v1/crates/<name>/owners`**
- [**Download crate archive**](crates/download/get.md): **`GET /api/v1/crates/<name>/<version>/download`**

**Categories section:**

- [**List crate categories**](categories/get.md): **`GET /api/v1/categories`**

**Account management section:**

- [**Login**](account/login/post.md): **`POST /api/v1/account/login`**
- [**Register**](account/register/post.md): **`POST /api/v1/account/register`**

Authenticated Endpoints
-----------------------

The following endpoints require a valid token to be specified in the `Authorization` request header.  
Refer to the [**Authentication docs**](authentication.md) to learn how to get one and how to use it.  

**Crates section:**

- [**Publish crate**](crates/publish/put.md): **`PUT /api/v1/crates/new`**
- [**Add crate owners**](crates/owners/put.md): **`PUT /api/v1/crates/<name>/owners`**
- [**Remove crate owners**](crates/owners/delete.md): **`DELETE /api/v1/crates/<name>/owners`**
- [**Yank crate version**](crates/yank/delete.md): **`DELETE /api/v1/crates/<name>/<version>/yank`**
- [**Unyanking crate version**](crates/unyank/put.md): **`PUT /api/v1/crates/<name>/<version>/unyank`**

**Account management section:**

- [**Get token information (from name)**](account/tokens/get.md): **`GET /api/v1/account/tokens/<name>`**
- [**Get token information (from token)**](account/tokens/post.md): **`POST /api/v1/account/tokens`**
- [**Generate authentication token**](account/tokens/put.md): **`PUT /api/v1/account/tokens`**
- [**Revoke authentication token**](account/tokens/delete.md): **`DELETE /api/v1/account/tokens`**
