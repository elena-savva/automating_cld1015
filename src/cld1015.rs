use std::io::{self};
use visa_rs::prelude::*;

pub fn io_to_vs_err(err: std::io::Error) -> visa_rs::Error {
    visa_rs::io_to_vs_err(err)
}