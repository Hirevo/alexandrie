use crate::db::models::Author;

/// Returns the number of authentication methods available for this crate author.
pub fn count_auth_methods(author: &Author) -> usize {
    let mut count = 0;

    if author.passwd.is_some() {
        count += 1;
    }

    if author.github_id.is_some() {
        count += 1;
    }

    if author.gitlab_id.is_some() {
        count += 1;
    }

    count
}
