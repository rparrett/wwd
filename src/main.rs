#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate toml;
extern crate darksky;
extern crate hyper;
extern crate hyper_native_tls;
extern crate chrono;

extern crate iron;
extern crate router;
extern crate logger;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate handlebars_iron as hbs;
extern crate mount;
extern crate staticfile;

extern crate tokio_timer;
extern crate tokio_core;
extern crate futures;

use std::path::Path;
use std::time::*;
use std::env;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;
use env_logger::LogBuilder;
use chrono::Local;
use hbs::{Template, HandlebarsEngine, DirectorySource};
use mount::Mount;
use staticfile::Static;
use config::Config;
use forecaster::{Forecaster, BasicWeekendForecast};

use tokio_timer::*;
use futures::*;
use tokio_core::reactor::Core;

use chrono::{DateTime, Utc};

mod helper;
mod forecaster;
mod config;

fn main() {
    let mut builder = LogBuilder::new();

    builder.format(|record| {
        format!(
            "{} [{}] - {}",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            record.args()
        )
    });

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();

    let mut core = Core::new().unwrap();

    let config = Config::new("config.toml").expect("Failed to open config file.");

    let (logger_before, logger_after) = Logger::new(None);

    let mut hbse = HandlebarsEngine::new();
    hbse.handlebars_mut().register_helper(
        "round",
        Box::new(helper::round),
    );
    hbse.handlebars_mut().register_helper(
        "color",
        Box::new(helper::color),
    );
    hbse.handlebars_mut().register_helper(
        "time_diff_in_words",
        Box::new(helper::time_diff_in_words),
    );
    hbse.add(Box::new(DirectorySource::new("templates", ".hbs")));

    hbse.reload().unwrap();

    let mut forecaster = Forecaster::new(config.darksky.unwrap(), config.locations.unwrap());
    match forecaster.get() {
        Ok(f) => f,
        Err(_) => error!("Failed to retrieve forecast."),
    };

    // we're cloning forecaster, which doesn't clone the underlying
    // reference counted / locked fields.

    let mut interval_forecaster = forecaster.clone();

    // with default settings, timer will panic with
    // long intervals. we can either increase max_timeout,
    // tick_duration, or num_slots. In this case, we don't
    // mind lowering the resolution to 1 second.

    let interval = wheel()
        .tick_duration(Duration::new(1, 0))
        .build()
        .interval(Duration::new(60 * 60, 0))
        .for_each(move |_| {
            match interval_forecaster.get() {
                Ok(f) => f,
                Err(_) => error!("Failed to retrieve forecast."),
            };

            Ok(())
        });

    let mut router = Router::new();
    router.get("/", move |r: &mut Request| index(r, &forecaster), "index");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static/", Static::new(Path::new("static")));

    let mut chain = Chain::new(mount);

    chain.link_before(logger_before);
    chain.link_after(hbse);
    chain.link_after(logger_after);

    fn index(_: &mut Request, forecaster: &Forecaster) -> IronResult<Response> {
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

    let _server = Iron::new(chain)
        .http(config.http.clone().unwrap().addr.unwrap())
        .unwrap();

    info!("Listening on {}", config.http.unwrap().addr.unwrap());

    core.run(interval).unwrap();
}
