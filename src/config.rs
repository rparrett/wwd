use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

use toml;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub secret: Option<String>,
    pub locations: Option<Vec<Location>>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Location {
    pub name: String,
    pub lat: f64,
    pub lon: f64
}

impl Config {
    pub fn new(filename: &str) -> Result<Config, Box<Error>> {
        let mut input = String::new();

        File::open(filename)?.read_to_string(&mut input)?;

        let config = toml::from_str(&input)?;

        Ok(config)
    }
}
