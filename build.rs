extern crate semver;
extern crate chrono;

use std::env::var;
use std::process::Command;
use semver::Version;
use chrono::NaiveDate;

fn main() {
    let unprefixed_jemalloc_version: Version = "1.8.0-dev".parse().unwrap();
    let unprefixed_jemalloc_day: NaiveDate = "2016-02-16".parse().unwrap();

    let rustc_version_string = Command::new(var("RUSTC").unwrap_or("rustc".into()))
        .arg("--version").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap();
    let rustc_version: Version = rustc_version_string.split_whitespace()
        .skip(1).next()
        .and_then(|r| r.parse().ok())
        .unwrap();
    let rustc_day: NaiveDate = rustc_version_string.split_whitespace()
        .last()
        .and_then(|r| r[..r.len()-1].parse().ok())
        .unwrap();

    if rustc_day < unprefixed_jemalloc_day
        || rustc_version < unprefixed_jemalloc_version
    {
        println!("cargo:rustc-cfg=prefixed_jemalloc");
    }
}
