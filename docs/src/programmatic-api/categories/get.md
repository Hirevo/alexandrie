Categories listing endpoint
===========================

This endpoint allows to list all the categories known to the registry.  

**Endpoint URL**: `/api/v1/categories`  
**HTTP Method**: `GET`  
**Endpoint Type:** Public  

Responses
---------

**Status:** `200 OK`

**Body:**

Currently, the registry will return an object of the following shape:

```js
{
    // Array of categories.
    "categories": [
        {
            // The name of the category.
            "name": "Development tools",
            // The tag of the category, used to refer to it from `Cargo.toml` files.
            "tag": "development-tools",
            // A brief description of the category.
            "description": "Crates that provide developer-facing features such as testing, debugging, linting, performance profiling, autocompletion, formatting, and more."
        }
    ],
    "meta": {
        // Total number of categories registered in the registry.
        "total": 71
    }
}
```
