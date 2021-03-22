use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use num_format::{Locale, ToFormattedString};

#[allow(unused)]
pub(crate) fn hbs_humanize_datetime(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .ok_or_else(|| RenderError::new("humanize_datetime: missing parameter"))?;
    let date = NaiveDateTime::from_str(param.value().as_str().unwrap()).map_err(|_| {
        RenderError::new("humanize_datetime: couldn't deserialize parameter into date")
    })?;
    out.write(humanize_datetime(date).as_str())?;
    Ok(())
}

#[allow(unused)]
pub(crate) fn hbs_humanize_date(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .ok_or_else(|| RenderError::new("humanize_date: missing parameter"))?;
    let date = NaiveDate::from_str(param.value().as_str().unwrap())
        .map_err(|_| RenderError::new("humanize_date: couldn't deserialize parameter into date"))?;
    out.write(humanize_date(date).as_str())?;
    Ok(())
}

#[allow(unused)]
pub(crate) fn hbs_humanize_number(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h
        .param(0)
        .ok_or_else(|| RenderError::new("humanize_number: missing parameter"))?;
    let formatted = param
        .value()
        .as_i64()
        .map(humanize_number)
        .or_else(|| param.value().as_i64().map(humanize_number))
        .ok_or_else(|| {
            RenderError::new("humanize_number: couldn't deserialize parameter into a number")
        })?;
    out.write(formatted.as_str())?;
    Ok(())
}

/// Humanizes a `NaiveDateTime` struct (to something like "Jul 18 2019, 18:19").
pub fn humanize_datetime(date: NaiveDateTime) -> String {
    date.format("%b %-d %Y, %H:%M").to_string()
}

/// Humanizes a `NaiveDate` struct (to something like "Jul 18 2019").
pub fn humanize_date(date: NaiveDate) -> String {
    date.format("%b %-d %Y").to_string()
}

/// Humanizes a number (to something like "432'523.452").
pub fn humanize_number(num: impl ToFormattedString) -> String {
    num.to_formatted_string(&Locale::en)
}
