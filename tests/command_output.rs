#![expect(unused_crate_dependencies)]
use std::process::{Command, Stdio};

fn run_cargo_trim(args: &[&str]) {
    let binary_program = env!("CARGO_BIN_EXE_cargo-trim");
    let status = Command::new(binary_program)
        .stdout(Stdio::null())
        .args(args)
        .status()
        .unwrap();
    assert!(status.success());
}

// test check trim subcommand is run currently and exit with status success
#[test]
fn command_line_success_test() {
    run_cargo_trim(&["help"]);
    run_cargo_trim(&["help", "list"]);
    run_cargo_trim(&["help", "config"]);
    run_cargo_trim(&["help", "git"]);
    run_cargo_trim(&["help", "registry"]);
    run_cargo_trim(&["help", "set"]);
    run_cargo_trim(&["help", "unset"]);
}
