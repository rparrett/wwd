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
