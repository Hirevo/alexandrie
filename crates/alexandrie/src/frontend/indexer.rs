use diesel::prelude::*;
use log::{info, warn};
use tide::{Request, Response};

use crate::db::models::Crate;
use crate::db::schema::*;
use crate::error::Error;
use crate::fts::TantivyDocument;
use crate::State;

const NUMBER_RESULT_PER_PAGE: i64 = 1000;

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let state = req.state().clone();
    let repo = &state.db;

    let transaction: Result<(), Error> = repo
        .transaction(move |conn| {
            let state = req.state();
            let mut tantivy = state.search.write().unwrap();
            tantivy.delete_all_documents()?;
            tantivy.commit()?;
            let mut start: i64 = 0;
            let mut count_crate = 0;

            loop {
                let krates = crates::table
                    .order_by(crates::id.asc())
                    .limit(NUMBER_RESULT_PER_PAGE)
                    .offset(start)
                    .load::<Crate>(conn)?;
                if krates.is_empty() {
                    info!("End indexing");
                    break;
                }

                let ids = krates
                    .clone()
                    .into_iter()
                    .map(|c| c.id)
                    .collect::<Vec<i64>>();
                start = start + krates.len() as i64;

                info!("Crates {:?}", ids);

                let keywords = keywords::table
                    .inner_join(crate_keywords::table)
                    .select((crate_keywords::crate_id, keywords::name))
                    .filter(crate_keywords::crate_id.eq_any(&ids))
                    .order_by(crate_keywords::crate_id.asc())
                    .load::<(i64, String)>(conn)?;

                let categories = categories::table
                    .inner_join(crate_categories::table)
                    .select((crate_categories::crate_id, categories::name))
                    .filter(crate_categories::crate_id.eq_any(&ids))
                    .order_by(crate_categories::crate_id.asc())
                    .load::<(i64, String)>(conn)?;

                let mut keywords_iterator = keywords.into_iter();
                let mut categories_iterator = categories.into_iter();

                let mut current_keyword: Option<(i64, String)> = keywords_iterator.next();
                let mut current_category: Option<(i64, String)> = categories_iterator.next();

                for krate in krates.into_iter() {
                    info!("crate {:?}", krate);
                    // Create a document with database ID and crate name
                    let mut doc: TantivyDocument =
                        TantivyDocument::new(krate.id, krate.name.clone(), &tantivy.schema);

                    // If there is some description, then set it
                    if let Some(description) = krate.description.as_ref() {
                        doc.set_description(description.clone());
                    }

                    // Add all keywords
                    while current_keyword.is_some()
                        && current_keyword.as_ref().unwrap().0 == krate.id
                    {
                        doc.add_keyword(current_keyword.unwrap().1);
                        current_keyword = keywords_iterator.next();
                    }

                    // Add all cateogries
                    while current_category.is_some()
                        && current_category.as_ref().unwrap().0 == krate.id
                    {
                        doc.add_category(current_category.unwrap().1);
                        current_category = categories_iterator.next();
                    }

                    // TODO get README

                    match doc.try_into() {
                        Ok(document) => {
                            tantivy.create_or_update(krate.id, document)?;
                        }
                        Err(error) => {
                            warn!(
                                "Can't convert crate '{}' ({}) into Tantivy document : {error}",
                                krate.id,
                                krate.name.clone()
                            );
                        }
                    }
                    count_crate += 1;
                }

                (*tantivy).commit()?;
                info!("{} crates indexed", count_crate);
            }

            Ok(())
        })
        .await;

    transaction?;
    let res = Response::new(200);
    Ok(res)
}
