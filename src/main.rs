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

use std::path::Path;

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

mod forecaster;
mod config;

fn main() {
    env_logger::init().unwrap();

    let config = Config::new("config.toml").expect("Failed to open config file.");

    let (logger_before, logger_after) = Logger::new(None);    

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("templates", ".hbs")));

    hbse.reload().unwrap();

    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/update", update, "update");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/static/", Static::new(Path::new("static")));

    let mut chain = Chain::new(mount);

    let mut forecaster = Forecaster::new(config);
    forecaster.get();

    chain.link_before(logger_before);
    chain.link_before(State::<Forecaster>::one(forecaster));
    chain.link_after(hbse);
    chain.link_after(logger_after);

    fn index(req: &mut Request) -> IronResult<Response> {
        let rwlock = req.get::<State<Forecaster>>().unwrap();
        let forecaster = rwlock.read().unwrap();

        #[derive(Serialize)]
        struct TemplateData {
            some_string: String,
            forecaster_cache: Vec<BasicWeekendForecast>,
            fetched: String,
            created: String,
        }

        let data = TemplateData {
            some_string: "test2".to_string(),
            forecaster_cache: forecaster.cache.clone(),
            fetched: forecaster.fetched.clone(),
            created: forecaster.created.clone()
        };

        let mut resp = Response::new();
        
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);

        Ok(resp)
    }

    fn update(req: &mut Request) -> IronResult<Response> {
        let rwlock = req.get::<State<Forecaster>>().unwrap();
        let mut forecaster = rwlock.write().unwrap();

        forecaster.get();

        Ok(Response::with((iron::status::Ok, "Updated")))
    }

    let _server = Iron::new(chain).http("96.126.101.191:3000").unwrap();
    
    println!("on 3000");
}
