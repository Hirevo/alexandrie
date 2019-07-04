
use rocket_contrib::json::Json;
use json::{json, Value};

#[catch(404)]
pub fn catch_404() -> Json<Value> {
    Json(json!({
        "errors": [{
            "detail": "resource not found",
        }]
    }))
}

#[catch(500)]
pub fn catch_500() -> Json<Value> {
    Json(json!({
        "errors": [{
            "detail": "internal server error",
        }]
    }))
}

#[catch(401)]
pub fn catch_401() -> Json<Value> {
    Json(json!({
        "errors": [{
            "detail": "access forbidden",
        }]
    }))
}
