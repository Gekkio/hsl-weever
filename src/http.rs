use chrono::{Local, TimeZone};
use hyper;
use hyper::{Client, Url};
use regex::Regex;
use rustc_serialize::json;
use std::io::Read;
use url::percent_encoding::{DEFAULT_ENCODE_SET, utf8_percent_encode};

use ::{Departure, RequestConfig};
use ::error::BusError;

#[derive(Debug, RustcDecodable)]
struct JsonStopPattern {
    pattern: JsonPattern,
    times: Vec<JsonStopTime>,
}

#[derive(Debug, RustcDecodable)]
struct JsonPattern {
    id: String,
}

#[allow(non_snake_case)]
#[derive(Debug, RustcDecodable)]
struct JsonStopTime {
    serviceDay: i64,
    scheduledArrival: i64,
    tripId: String,
}

static API_V1: &'static str = "http://api.digitransit.fi/routing/v1";

fn encode(input: &str) -> String { utf8_percent_encode(input, DEFAULT_ENCODE_SET).collect() }

fn build_url(config: &RequestConfig, code: &str) -> Result<Url, BusError> {
    let url_str = format!("{}/routers/hsl/index/stops/HSL:{}/stoptimes", API_V1, encode(code));
    let mut url = Url::parse(&url_str)?;

    if let Some(value) = config.departures_per_pattern {
        let mut query_params = url.query_pairs_mut();
        query_params.append_pair("numberOfDepartures", &format!("{}", value));
    }

    Ok(url)
}

pub fn fetch_stop_departures(client: &Client,
                             config: &RequestConfig,
                             code: &str)
                             -> Result<Vec<Departure>, BusError> {
    let url = build_url(config, code)?;

    let mut res = client.get(url).send()?;
    if res.status != hyper::Ok {
        return Err(BusError(format!("HTTP error for stop {}: {}", code, res.status).into()));
    }

    let mut s = String::new();
    res.read_to_string(&mut s)?;
    let responses: Vec<JsonStopPattern> = json::decode(&s)?;

    let pattern_id_re = Regex::new(r"^HSL:[0-9]0*([0-9A-Z]+):[0-9]+:[0-9]+$")?;

    let mut departures = vec![];

    for stop_pattern in responses {
        let pattern_match = pattern_id_re.captures(&stop_pattern.pattern.id)
            .ok_or(BusError("Pattern ID did not match".into()))?;
        let bus_code = pattern_match[1].to_owned();

        for time in stop_pattern.times {
            let secs = time.serviceDay + time.scheduledArrival;
            let nsecs = 0;
            let timestamp = Local.timestamp(secs, nsecs);

            departures.push(Departure {
                bus: bus_code.clone(),
                timestamp: timestamp,
            });
        }
    }
    Ok(departures)
}
