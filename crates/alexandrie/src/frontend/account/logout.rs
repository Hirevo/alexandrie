use axum::response::Redirect;
use tower_sessions::Session;

pub(crate) async fn get(mut session: Session) -> Redirect {
    session.remove_value("author.id");
    Redirect::to("/")
}
