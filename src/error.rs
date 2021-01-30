use chrono;
use hyper;
use regex;
use rustc_serialize;
use std::borrow::Cow;
use std::io;

#[derive(Debug)]
pub struct BusError(pub Cow<'static, str>);

impl From<chrono::format::ParseError> for BusError {
    fn from(error: chrono::format::ParseError) -> BusError {
        BusError(format!("Chrono: {}", error).into())
    }
}

impl From<hyper::error::Error> for BusError {
    fn from(error: hyper::error::Error) -> BusError {
        BusError(format!("Hyper: {}", error).into())
    }
}

impl From<io::Error> for BusError {
    fn from(error: io::Error) -> BusError {
        BusError(format!("IO: {}", error).into())
    }
}

impl From<regex::Error> for BusError {
    fn from(error: regex::Error) -> BusError {
        BusError(format!("Regex: {}", error).into())
    }
}

impl From<rustc_serialize::json::DecoderError> for BusError {
    fn from(error: rustc_serialize::json::DecoderError) -> BusError {
        BusError(format!("JSON: {}", error).into())
    }
}

impl From<hyper::error::ParseError> for BusError {
    fn from(error: hyper::error::ParseError) -> BusError {
        BusError(format!("Parse: {}", error).into())
    }
}
