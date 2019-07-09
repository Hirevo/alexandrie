pub fn humanize_datetime(date: chrono::NaiveDateTime) -> String {
    date.format("%b %-d %Y, %H:%M").to_string()
}

pub fn humanize_date(date: chrono::NaiveDate) -> String {
    date.format("%b %-d %Y").to_string()
}
