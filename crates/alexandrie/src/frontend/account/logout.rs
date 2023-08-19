use axum::response::Redirect;
use axum_sessions::extractors::WritableSession;

pub(crate) async fn get(mut session: WritableSession) -> Redirect {
    session.remove("author.id");
    Redirect::to("/")
}
