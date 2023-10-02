use std::fmt;

use time::format_description;
// use async_session::{Session, SessionStore};
use axum::async_trait;
use diesel::dsl;
use diesel::prelude::*;
use thiserror::Error;
use time::macros::format_description;
use tower_sessions::session::{Session, SessionId, SessionRecord};
use tower_sessions::session_store::SessionStore;

use crate::db::models::Session as SessionEntry;
use crate::db::schema::*;
use crate::db::Database;

/// The date-time format used in the database.
pub static SESSION_DATE_FORMAT: &[format_description::FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

/// The error type when interacting with the session store.
#[derive(Debug, Error)]
pub enum SqlStoreError {
    /// Session error.
    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::SessionError),
    /// JSON (de)serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] json::Error),
    /// Time formatting error.
    #[error("time formatting error: {0}")]
    TimeFormat(#[from] time::error::Format),
    /// Time parsing error.
    #[error("time parsing error: {0}")]
    TimeParse(#[from] time::error::Parse),
    /// Diesel query error.
    #[error("diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),
}

/// A SQL-based session store.
#[derive(Clone)]
pub struct SqlStore {
    database: Database,
}

impl SqlStore {
    /// Create a new `SqlStore`.
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl fmt::Debug for SqlStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionStorage").finish()
    }
}

#[async_trait]
impl SessionStore for SqlStore {
    type Error = SqlStoreError;

    async fn save(&self, session: &SessionRecord) -> Result<(), Self::Error> {
        let id = session.id().to_string();
        let data = session.data();
        let author_id: Option<i64> = data
            .get("author.id")
            .cloned()
            .map(json::from_value)
            .transpose()?;

        let data = json::to_string(&data)?;

        let expiry = session
            .expiration_time()
            .map(|it| it.format(SESSION_DATE_FORMAT))
            .transpose()?
            .unwrap_or_default();

        let record = SessionEntry {
            id,
            author_id,
            expiry,
            data,
        };

        self.database
            .transaction(move |conn| -> Result<(), Self::Error> {
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

        Ok(())
    }

    async fn load(&self, session_id: &SessionId) -> Result<Option<Session>, Self::Error> {
        let id = session_id.to_string();
        let now = time::OffsetDateTime::now_utc().format(SESSION_DATE_FORMAT)?;

        let maybe_entry: Option<SessionEntry> = self
            .database
            .run(|conn| {
                sessions::table
                    .find(id)
                    .filter(sessions::expiry.gt(now))
                    .first(conn)
                    .optional()
            })
            .await?;

        let Some(entry) = maybe_entry else {
            return Ok(None);
        };

        let session_id = SessionId::try_from(entry.id)?;
        let expiration_time =
            time::PrimitiveDateTime::parse(&entry.expiry, SESSION_DATE_FORMAT)?.assume_utc();
        let data = json::from_str(&entry.data)?;
        let session_record = SessionRecord::new(session_id, Some(expiration_time), data);

        Ok(Some(session_record.into()))
    }

    async fn delete(&self, session_id: &SessionId) -> Result<(), Self::Error> {
        let id = session_id.to_string();
        self.database
            .run(move |conn| diesel::delete(sessions::table.find(id)).execute(conn))
            .await?;
        Ok(())
    }
}
