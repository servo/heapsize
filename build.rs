extern crate regex;

use std::env::var;
use std::process::Command;
use std::str;

fn main() {
    let version_line = Command::new(var("RUSTC").unwrap_or("rustc".into()))
        .arg("--version")
        .output()
        .unwrap()
        .stdout;
    let captures = regex::Regex::new(r"rustc (\d+)\.(\d+)\.(\d+).+(\d{4}-\d{2}-\d{2})\)")
        .unwrap()
        .captures(str::from_utf8(&version_line).unwrap())
        .unwrap();
    let version = (
        captures[1].parse::<u32>().unwrap(),
        captures[2].parse::<u32>().unwrap(),
        captures[3].parse::<u32>().unwrap(),
        &captures[4],
    );
    if version < (1, 8, 0, "2016-02-14") {
        println!("cargo:rustc-cfg=prefixed_jemalloc");
    }
}
