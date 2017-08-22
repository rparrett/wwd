#[macro_use]
extern crate serde_derive;

extern crate toml;
extern crate darksky;
extern crate hyper;
extern crate hyper_native_tls;
extern crate chrono;

extern crate iron;
extern crate router;
extern crate logger;
extern crate env_logger;
extern crate handlebars_iron as hbs;
extern crate mount;
extern crate staticfile;
extern crate persistent;

extern crate tokio_timer;
extern crate tokio_core;
extern crate futures;

use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::*;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;
use hbs::{Template, HandlebarsEngine, DirectorySource};
use mount::Mount;
use staticfile::Static;
use persistent::{State};
use config::Config;
use forecaster::{Forecaster, BasicWeekendForecast};

use tokio_timer::*;
use futures::*;
use tokio_core::reactor::Core;

mod forecaster;
mod config;

fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();
    let timer = Timer::default();

    let config = Config::new("config.toml").expect("Failed to open config file.");

    let (logger_before, logger_after) = Logger::new(None);    

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("templates", ".hbs")));

    hbse.reload().unwrap();

    let mut forecaster = Forecaster::new(
        config.darksky.unwrap(), 
        config.locations.unwrap()
    );
    forecaster.get();

    let mut interforecaster = forecaster.clone();

    let interval = 
        Timer::default()
        .interval(Duration::new(60 * 60, 0))
        .for_each(move |_| {
            interforecaster.get();

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

    fn index(req: &mut Request, forecaster: &Forecaster) -> IronResult<Response> {
        #[derive(Serialize)]
        struct TemplateData {
            some_string: String,
            forecaster_cache: Vec<BasicWeekendForecast>,
            fetched: String,
            created: String,
        }

        let data = TemplateData {
            some_string: "test2".to_string(),
            forecaster_cache: forecaster.cache.read().unwrap().clone(),
            fetched: forecaster.fetched.read().unwrap().clone(),
            created: forecaster.created.clone()
        };

        let mut resp = Response::new();
        
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);

        Ok(resp)
    }

    let _server = Iron::new(chain).http(config.http.unwrap().addr.unwrap()).unwrap();
    
    println!("Listening on 3000");
    
    core.run(interval).unwrap();
}
