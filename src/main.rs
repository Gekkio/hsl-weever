extern crate chrono;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate regex;
extern crate rustc_serialize;
extern crate url;

use chrono::{DateTime, Local, Timelike};
use clap::{App, Arg};
use hyper::Client;
use regex::Regex;
use std::collections::HashSet;
use std::io::Write;
use std::process;

use crate::error::BusError;

mod error;
mod http;

fn fetch_departures(config: RequestConfig, stops: Vec<&str>) -> Result<Vec<Departure>, BusError> {
    let client = Client::new();

    let mut seen_buses = HashSet::new();
    let mut departures = vec![];
    for stop in stops {
        let stop_departures = http::fetch_stop_departures(&client, &config, &stop)?;
        let mut stop_buses = HashSet::new();
        for departure in stop_departures {
            if seen_buses.contains(&departure.bus) {
                continue;
            }
            stop_buses.insert(departure.bus.clone());
            departures.push(departure);
        }
        seen_buses.extend(stop_buses);
    }

    departures.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(departures)
}

#[derive(Debug)]
pub struct RequestConfig {
    departures_per_pattern: Option<u32>,
}

#[derive(Debug)]
pub struct Departure {
    bus: String,
    timestamp: DateTime<Local>,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run_main() -> Result<(), BusError> {
    let stop_id_re = Regex::new(r"^[0-9]+$")?;
    let matches = App::new("hsl-weever")
        .version(VERSION)
        .author("Joonas Javanainen <joonas.javanainen@gmail.com>")
        .about("A utility for fetching HSL bus departure data for stops")
        .arg(Arg::with_name("DEPARTURES_PER_PATTERN")
            .help("Departures to fetch per trip pattern")
            .takes_value(true)
            .short("d")
            .long("departures-per-pattern"))
        .arg(Arg::with_name("STOP_ID")
            .help("HSL stop IDs (descending priority)")
            .multiple(true)
            .required(true)
            .validator(move |v| {
                if stop_id_re.is_match(&v) {
                    Ok(())
                } else {
                    Err("Invalid stop ID".to_owned())
                }
            })
            .index(1))
        .get_matches();

    let stops = matches.values_of("STOP_ID").unwrap().collect();
    let request_config = RequestConfig {
        departures_per_pattern: if matches.is_present("DEPARTURES_PER_PATTERN") {
            Some(value_t!(matches, "DEPARTURES_PER_PATTERN", u32).unwrap_or_else(|e| e.exit()))
        } else {
            None
        },
    };

    let departures = fetch_departures(request_config, stops)?;
    let mut stdout = std::io::stdout();
    for d in departures {
        let _ = writeln!(&mut stdout,
                         "{:02}:{:02}\t{}",
                         d.timestamp.hour(),
                         d.timestamp.minute(),
                         d.bus);
    }
    Ok(())
}

fn main() {
    if let Err(error) = run_main() {
        let mut stderr = std::io::stderr();
        let _ = writeln!(&mut stderr, "{}", error.0);
        process::exit(1);
    }
}
