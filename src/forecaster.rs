use darksky::*;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper_native_tls::NativeTlsClient;

use chrono::{DateTime, Utc, Weekday, Datelike, TimeZone};

use iron::typemap::Key;

use config::DarkskyConfig;
use config::Location;

use std::sync::{Arc, RwLock};

#[inline]
fn client() -> Client {
    let tc = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(tc);

    Client::with_connector(connector)
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeekendForecast {
    location: Location,
    days: Vec<BasicWeather>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeather {
    time: String,
    temperature_low: f64,
    temperature_high: f64,
    summary: String,
}

#[derive(Clone)]
pub struct Forecaster {
    pub config: DarkskyConfig,
    pub locations: Vec<Location>,
    pub created: DateTime<Utc>,
    pub cache: Arc<RwLock<Vec<BasicWeekendForecast>>>,
    pub fetched: Arc<RwLock<DateTime<Utc>>>,
}

impl Key for Forecaster {
    type Value = Forecaster;
}

impl Forecaster {
    pub fn new(config: DarkskyConfig, locations: Vec<Location>) -> Forecaster {
        Forecaster {
            config: config,
            locations: locations,
            cache: Arc::new(RwLock::new(Vec::new())),
            created: Utc::now(),
            fetched: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub fn get(&mut self) -> Result<()> {
        info!("Starting fetch.");

        let config = self.config.clone();

        let secret: String = config.secret.unwrap();

        let client = client();

        let mut data = Vec::new();

        for x in &self.locations {
            info!("Fetching {}", x.name);

            let mut weekend = BasicWeekendForecast {
                location: x.clone(),
                days: Vec::new(),
            };

            let f = client.get_forecast(&secret, x.lat, x.lon)?;

            for d in f.daily.unwrap().data.unwrap() {
                let dt = Utc.timestamp(d.time as i64, 0);

                // Is this gross?

                match dt.weekday() {
                    Weekday::Fri | Weekday::Sat | Weekday::Sun => {}
                    _ => continue,
                }

                let weather = BasicWeather {
                    time: dt.format("%a %h %e").to_string(),
                    temperature_low: d.temperature_low.unwrap(),
                    temperature_high: d.temperature_high.unwrap(),
                    summary: d.summary.unwrap(),
                };

                weekend.days.push(weather);
            }

            data.push(weekend)
        }

        info!("Updating cache");

        let mut fetched = self.fetched.write().unwrap();
        *fetched = Utc::now();

        let mut cache = self.cache.write().unwrap();

        cache.clear();
        for i in &data {
            cache.push((*i).clone());
        }

        Ok(())
    }
}
