use darksky::*;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper_native_tls::NativeTlsClient;

use chrono::{DateTime, NaiveDateTime, Utc, Weekday, Datelike};

use iron::typemap::Key;

use config::Config;
use config::Location;

#[inline]
fn client() -> Client {
    let tc = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(tc);

    Client::with_connector(connector)
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeekendForecast {
    location: Location,
    days: Vec<BasicWeather>
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicWeather {
    time: String,
    temperature_min: f64,
    temperature_max: f64,
    summary: String
}

#[derive(Clone)]
pub struct Forecaster {
    pub config: Config,
    pub cache : Vec<BasicWeekendForecast>,
    pub created: String,
    pub fetched: String
}

impl Key for Forecaster {
    type Value = Forecaster;
}

impl Forecaster {
    pub fn new(config: Config) -> Forecaster {
        Forecaster {
            config: config,
            cache: Vec::new(),
            created: Utc::now().to_string(),
            fetched: "".to_string()
        }
    }

    pub fn get(&mut self) {
        let config = self.config.clone();

        let secret: String = config.secret.unwrap();
        
        let client = client();

        self.cache.clear();

        for x in &config.locations.unwrap() {
            let mut weekend = BasicWeekendForecast {
                location: x.clone(),
                days: Vec::new()
            };

            let f = client.get_forecast(&secret, x.lat, x.lon).unwrap();
            
            println!(
                "{}:",
                x.name
            );

            for d in f.daily.unwrap().data.unwrap() {
                let dt = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp(d.time as i64, 0), 
                    Utc
                );

                // Is this gross?

                match dt.weekday() {
                    Weekday::Fri | Weekday::Sat | Weekday::Sun => {}
                    _ => continue
                }

                let weather = BasicWeather {
                    time: dt.format("%a %h %e").to_string(), 
                    temperature_min: d.temperature_min.unwrap(),
                    temperature_max: d.temperature_max.unwrap(),
                    summary: d.summary.unwrap()
                };

                weekend.days.push(weather);
            }

            self.fetched = Utc::now().to_string();

            self.cache.push(weekend)
        }
    }
}
