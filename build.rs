extern crate chrono;

use chrono::UTC;

use std::process::Command;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;

fn main() {
    // Rustc version
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let rustc_version = Command::new(&rustc)
        .arg("--version")
        .output()
        .into_iter()
        .flat_map(|out| String::from_utf8(out.stdout))
        .next()
        .unwrap_or("rustc unknown-version".into());

    // Git SHA
    let git = env::var_os("GIT").unwrap_or_else(|| OsString::from("git"));
    let git_sha = Command::new(&git)
        .args(&vec!["rev-parse", "--short", "HEAD"])
        .output()
        .into_iter()
        .flat_map(|out| String::from_utf8(out.stdout))
        .next()
        .unwrap_or("no-SHA".into());


    // Write it all to version.rs
    let mut f = File::create("src/version.rs").unwrap();
    write!(f,
r#"
pub static VERSION: &'static str =
    "algo {pkg_ver} ({git_sha} {build_date})\n{rustc_version}";
"#,
        pkg_ver = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown-version"),
        git_sha = git_sha.trim(),
        build_date = UTC::today().format("%Y-%m-%d"),
        rustc_version = rustc_version.trim(),
    ).unwrap();
}