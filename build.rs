extern crate semver;

use std::env::var;
use std::process::Command;
use semver::Version;

fn main() {
    let unprefixed_jemalloc_version: Version = "1.8.0-dev".parse().unwrap();
    let rustc_version: Version = Command::new(var("RUSTC").unwrap_or("rustc".into()))
        .arg("--version").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().skip(1).next().map(|r| r.to_string())).unwrap()
        .parse().unwrap();

    if rustc_version < unprefixed_jemalloc_version {
        println!("cargo:rustc-cfg=prefixed_jemalloc");
    }
}
