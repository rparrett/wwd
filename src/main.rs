#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate chrono;
extern crate darksky;
extern crate hyper;
extern crate hyper_native_tls;
extern crate toml;

extern crate iron;
extern crate logger;
extern crate router;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate handlebars_iron as hbs;
extern crate mount;
extern crate staticfile;

extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

use std::env;
use std::path::Path;
use std::time::*;

use chrono::Local;
use config::Config;
use env_logger::LogBuilder;
use forecaster::Forecaster;
use hbs::{DirectorySource, HandlebarsEngine};
use iron::prelude::*;
use logger::Logger;
use mount::Mount;
use router::Router;
use staticfile::Static;

use futures::*;
use tokio_core::reactor::Core;
use tokio_timer::*;

mod config;
mod forecaster;
mod handlers;
mod helper;

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
    hbse.handlebars_mut()
        .register_helper("round", Box::new(helper::round));
    hbse.handlebars_mut()
        .register_helper("color", Box::new(helper::color));
    hbse.handlebars_mut()
        .register_helper("time_diff_in_words", Box::new(helper::time_diff_in_words));
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
    router.get(
        "/",
        move |r: &mut Request| handlers::get_index(r, &forecaster),
        "index",
    );

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static/", Static::new(Path::new("static")));

    let mut chain = Chain::new(mount);

    chain.link_before(logger_before);
    chain.link_after(hbse);
    chain.link_after(logger_after);

    let _server = Iron::new(chain)
        .http(config.http.clone().unwrap().addr.unwrap())
        .unwrap();

    info!("Listening on {}", config.http.unwrap().addr.unwrap());

    core.run(interval).unwrap();
}
