use iron::prelude::*;
use iron::status;

use chrono::{DateTime, Utc};

use forecaster::{Forecaster, BasicWeekendForecast};

use hbs::Template;

pub fn get_index(_: &mut Request, forecaster: &Forecaster) -> IronResult<Response> {
    #[derive(Serialize)]
    struct TemplateData {
        forecaster_cache: Vec<BasicWeekendForecast>,
        fetched: DateTime<Utc>,
        now: DateTime<Utc>,
    }

    let data = TemplateData {
        forecaster_cache: forecaster.cache.read().unwrap().clone(),
        fetched: forecaster.fetched.read().unwrap().clone(),
        now: Utc::now(),
    };

    let mut resp = Response::new();

    resp.set_mut(Template::new("index", data)).set_mut(
        status::Ok,
    );

    Ok(resp)
}
