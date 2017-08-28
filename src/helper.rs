use chrono::{DateTime, Utc};

use serde_json;

use hbs::handlebars::{Helper, RenderContext, RenderError, Handlebars};

pub fn round(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.param(0).unwrap().value();
    let rendered = format!("{:.0}", param.as_f64().unwrap().round());
    try!(rc.writer.write(rendered.into_bytes().as_ref()));

    Ok(())
}

pub fn color(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.param(0).unwrap().value();
    let f = param.as_f64().unwrap().round();

    let color = if f < 32.0 {
        "white"
    } else if f < 68.0 {
        "cyan"
    } else if f < 79.0 {
        "#99ff00"
    } else if f < 90.0 {
        "orange"
    } else {
        "red"
    };

    try!(rc.writer.write(color.as_ref()));

    Ok(())
}

pub fn time_diff_in_words(
    h: &Helper,
    _: &Handlebars,
    rc: &mut RenderContext,
) -> Result<(), RenderError> {
    let time: DateTime<Utc> = serde_json::from_value(h.param(0).unwrap().value().clone()).unwrap();
    let now: DateTime<Utc> = serde_json::from_value(h.param(1).unwrap().value().clone()).unwrap();

    let diff = now.signed_duration_since(time);

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();

    let rendered = if minutes < 1 {
        "less than 1 minute".to_string()
    } else if minutes == 1 {
        "1 minute".to_string()
    } else if minutes < 60 {
        format!("{} minutes", minutes)
    } else if hours == 1 {
        "1 hour".to_string()
    } else {
        format!("{} hours", hours)
    };

    try!(rc.writer.write(rendered.into_bytes().as_ref()));

    Ok(())
}
