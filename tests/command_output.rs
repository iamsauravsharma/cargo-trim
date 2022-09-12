use std::process::{Command, Stdio};

fn run_cargo_trim(args: &[&str]) {
    let status = Command::new("cargo")
        .stdout(Stdio::null())
        .arg("run")
        .arg("--")
        .arg("trim")
        .args(args)
        .status()
        .unwrap();
    assert!(status.success());
}

// test check trim subcommand help
#[test]
fn test_help() {
    run_cargo_trim(&["help"]);
}

// test check list subcommand help
#[test]
fn test_list_help() {
    run_cargo_trim(&["help", "list"]);
}

// test check config subcommand help
#[test]
fn test_config_help() {
    run_cargo_trim(&["help", "config"]);
}

// test check git subcommand help
#[test]
fn test_git_help() {
    run_cargo_trim(&["help", "git"]);
}

// test check registry subcommand help
#[test]
fn test_registry_help() {
    run_cargo_trim(&["help", "registry"]);
}

// test check set subcommand help
#[test]
fn test_set_help() {
    run_cargo_trim(&["help", "set"]);
}

// test check unset subcommand help
#[test]
fn test_unset_help() {
    run_cargo_trim(&["help", "unset"]);
}
