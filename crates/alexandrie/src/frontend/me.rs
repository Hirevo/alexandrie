use axum::response::Redirect;

pub(crate) async fn get() -> Redirect {
    Redirect::to("/account/manage")
}
