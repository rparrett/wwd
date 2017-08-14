extern crate darksky;
extern crate hyper;
extern crate hyper_native_tls;
extern crate chrono;
extern crate termion;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::fmt;

use darksky::*;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper_native_tls::NativeTlsClient;

use chrono::{DateTime, NaiveDateTime, Utc, Weekday, Datelike};

use termion::color;

#[derive(Debug, Deserialize)]
struct Config {
    secret: Option<String>,
    locations: Option<Vec<Location>>
}

#[derive(Debug, Deserialize)]
struct Location {
    name: String,
    lat: f64,
    lon: f64
}

#[inline]
fn client() -> Client {
    let tc = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(tc);

    Client::with_connector(connector)
}

struct Temperature(f64);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 40.0 {
            write!(f, "{}{:.0}{}", color::Fg(color::Blue), self.0, color::Fg(color::Reset))
        } else if self.0 < 68.0 {
            write!(f, "{}{:.0}{}", color::Fg(color::Cyan), self.0, color::Fg(color::Reset))
        } else if self.0 < 85.0 { 
            write!(f, "{}{:.0}{}", color::Fg(color::Green), self.0, color::Fg(color::Reset))
        } else {
            write!(f, "{}{:.0}{}", color::Fg(color::Red), self.0, color::Fg(color::Reset))
        }
    }
}

fn main() {
    let mut input = String::new();

    File::open("config.toml").and_then(|mut f| {
        f.read_to_string(&mut input)
    }).unwrap();

    let decoded: Config = toml::from_str(&input).unwrap();
    
    let secret: String = decoded.secret.unwrap();
    
    let client = client();

    for x in &decoded.locations.unwrap() {
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

            let temperature_min = Temperature(d.temperature_min.unwrap());
            let temperature_max = Temperature(d.temperature_max.unwrap());
            let summary = d.summary.unwrap();

            println!(
                "    {}: high {} low {} / {}", 
                dt.format("%a %h %e"), 
                temperature_max, 
                temperature_min, 
                summary
            );
        }
    }
}
