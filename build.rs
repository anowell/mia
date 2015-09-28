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
    let rustc_out = Command::new(&rustc)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    let rustc_version = String::from_utf8(rustc_out.stdout).unwrap();


    // Git SHA
    let git = env::var_os("GIT").unwrap_or_else(|| OsString::from("git"));
    let git_out = Command::new(&git)
        .args(&vec!["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    let git_sha = String::from_utf8(git_out.stdout).unwrap();


    // Write it all to version.rs
    let mut f = File::create("src/version.rs").unwrap();
    write!(f,
r#"
pub fn get_version() -> &'static str {{
    "algo {pkg_ver} ({git_sha} {build_date})\n{rustc_version}"
}}
"#,
        pkg_ver = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version"),
        git_sha = git_sha.trim(),
        build_date = UTC::today().format("%Y-%m-%d"),
        rustc_version = rustc_version.trim(),
    ).unwrap();
}