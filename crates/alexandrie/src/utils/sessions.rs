use std::fmt;

use async_session::{Session, SessionStore};
use axum::async_trait;
use diesel::dsl;
use diesel::prelude::*;

use crate::db::models::Session as SessionRecord;
use crate::db::Database;
use crate::db::{schema::*, DATETIME_FORMAT};

/// A SQL-based session store.
#[derive(Clone)]
pub struct SqlStore {
    db: Database,
}

impl SqlStore {
    /// Create a new `SqlStore`.
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl fmt::Debug for SqlStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionStorage").finish()
    }
}

#[async_trait]
impl SessionStore for SqlStore {
    async fn load_session(&self, cookie_value: String) -> async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        // eprintln!("searching session: {:?}", id);

        let data: Option<String> = self
            .db
            .run(|conn| {
                sessions::table
                    .find(id)
                    .select(sessions::data)
                    .first(conn)
                    .optional()
            })
            .await?;

        let session: Option<Session> = data.map(|data| json::from_str(&data)).transpose()?;
        Ok(session)
    }

    async fn store_session(&self, session: Session) -> async_session::Result<Option<String>> {
        let id = session.id().to_string();
        let data = json::to_string(&session)?;
        let author_id: Option<i64> = session.get("author.id");
        let expiry = session
            .expiry()
            .map(|it| it.format(DATETIME_FORMAT).to_string())
            .unwrap_or_default();

        let record = SessionRecord {
            id,
            author_id,
            expiry,
            data,
        };

        // eprintln!("storing session: {:?}", record);

        self.db
            .transaction(move |conn| -> async_session::Result<_> {
                let exists: bool =
                    dsl::select(dsl::exists(sessions::table.find(&record.id))).get_result(conn)?;

                if exists {
                    diesel::update(sessions::table.find(&record.id))
                        .set(&record)
                        .execute(conn)?;
                } else {
                    diesel::insert_into(sessions::table)
                        .values(&record)
                        .execute(conn)?;
                }

                Ok(())
            })
            .await?;

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> async_session::Result {
        self.db
            .run(move |conn| diesel::delete(sessions::table.find(session.id())).execute(conn))
            .await?;

        Ok(())
    }

    async fn clear_store(&self) -> async_session::Result {
        self.db
            .run(move |conn| diesel::delete(sessions::table).execute(conn))
            .await?;

        Ok(())
    }
}
