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
    run_cargo_trim(&["help", "list"]);
    run_cargo_trim(&["help", "config"]);
    run_cargo_trim(&["help", "git"]);
    run_cargo_trim(&["help", "registry"]);
    run_cargo_trim(&["help", "set"]);
    run_cargo_trim(&["help", "unset"]);
}
