use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::Category;
use crate::db::schema::*;
use crate::error::Error;
use crate::utils;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CategoriesResponse {
    pub categories: Vec<CategoriesResult>,
    pub meta: CategoriesMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CategoriesResult {
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
struct CategoriesMeta {
    pub total: usize,
}

/// Route to list categories.
pub(crate) async fn get(req: Request<State>) -> Result<Response, Error> {
    let state = req.state();
    let repo = &state.repo;

    let categories = repo
        .run(|conn| categories::table.load::<Category>(conn))
        .await?;

    let categories: Vec<_> = categories.into_iter().map(CategoriesResult::from).collect();
    let total = categories.len();

    let data = CategoriesResponse {
        categories,
        meta: CategoriesMeta { total },
    };
    Ok(utils::response::json(&data))
}
