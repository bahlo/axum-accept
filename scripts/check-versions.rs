#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! toml = "0.9"
//! ```
use std::{fs, path::PathBuf};
use toml::{Table, Value};

fn main() {
    let axum_accept = parse_cargo_toml("axum-accept/Cargo.toml");
    let axum_accept_macros = parse_cargo_toml("axum-accept-macros/Cargo.toml");
    let axum_accept_shared = parse_cargo_toml("axum-accept-shared/Cargo.toml");

    // check crate versions
    assert_eq!(
        axum_accept.version, axum_accept_macros.version,
        "axum-accept has a different version than axum-accept-macros"
    );
    assert_eq!(
        axum_accept_macros.version, axum_accept_shared.version,
        "axum-accept-macros has a different version than axum-accept-shared"
    );

    // check dependency versions
    for cargo in [&axum_accept, &axum_accept_macros, &axum_accept_shared] {
        if let Some(version) = &cargo.macros_version {
            assert_eq!(
                axum_accept_macros.version, *version,
                "axum-accept-macros has a different version than the dependency specified in {:?}",
                &cargo.path
            )
        }
        if let Some(version) = &cargo.shared_version {
            assert_eq!(
                axum_accept_shared.version, *version,
                "axum-accept-shared has a different version than the dependency specified in {:?}",
                &cargo.path
            )
        }
    }

    println!("✓ All versions match {}", axum_accept.version)
}

#[derive(PartialEq)]
struct CargoToml {
    path: PathBuf,
    version: String,
    macros_version: Option<String>,
    shared_version: Option<String>,
}

fn parse_cargo_toml(path: impl Into<PathBuf>) -> CargoToml {
    let path = path.into();
    let contents = fs::read_to_string(&path).expect("Failed to read Cargo.toml");
    let value = contents.parse::<Table>().expect("Failed to parse TOML");

    // read version
    let Value::Table(ref package) = value["package"] else {
        panic!("Expected package to be a Table");
    };
    let version = package["version"]
        .as_str()
        .expect("failed to find version")
        .to_string();

    // read dependency versions
    let Value::Table(ref dependencies) = value["dependencies"] else {
        panic!("Expected dependencies to be a Table");
    };
    let macros_version = match dependencies.get("axum-accept-macros") {
        Some(Value::Table(dependency)) => dependency["version"].as_str().map(|s| s.to_string()),
        _ => None,
    };
    let shared_version = match dependencies.get("axum-accept-shared") {
        Some(Value::Table(dependency)) => dependency["version"].as_str().map(|s| s.to_string()),
        _ => None,
    };

    CargoToml {
        path,
        version,
        macros_version,
        shared_version,
    }
}
