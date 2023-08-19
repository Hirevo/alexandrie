use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::models::Category;
use crate::db::schema::*;
use crate::error::ApiError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CategoriesResponse {
    pub categories: Vec<CategoriesResult>,
    pub meta: CategoriesMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CategoriesResult {
    pub name: String,
    pub tag: String,
    pub description: String,
}

impl From<Category> for CategoriesResult {
    fn from(category: Category) -> CategoriesResult {
        CategoriesResult {
            name: category.name,
            tag: category.tag,
            description: category.description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CategoriesMeta {
    pub total: usize,
}

/// Route to list categories.
pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CategoriesResponse>, ApiError> {
    let db = &state.db;

    let categories = db
        .run(|conn| categories::table.load::<Category>(conn))
        .await?;

    let categories: Vec<_> = categories.into_iter().map(CategoriesResult::from).collect();
    let total = categories.len();

    Ok(Json(CategoriesResponse {
        categories,
        meta: CategoriesMeta { total },
    }))
}
