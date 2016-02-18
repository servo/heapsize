extern crate chrono;

use std::env::var;
use std::process::Command;
use chrono::{ NaiveDate };


fn main() {
    let unprefixed_jemalloc_day: NaiveDate = "2016-02-16".parse().unwrap();
    let rustc_day: NaiveDate = Command::new(var("RUSTC").unwrap_or("rustc".into()))
        .arg("--version").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().last().map(|r| r[..r.len()-1].to_string())).unwrap()
        .parse().unwrap();

    if rustc_day < unprefixed_jemalloc_day {
        println!("cargo:rustc-cfg=prefixed_jemalloc");
    }
}
