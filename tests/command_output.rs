use std::process::Command;

// test check trim subcommand help
#[test]
fn test_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check list subcommand help
#[test]
fn test_list_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("list")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check config subcommand help
#[test]
fn test_config_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("config")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check git subcommand help
#[test]
fn test_git_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("git")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check registry subcommand help
#[test]
fn test_registry_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("registry")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check set subcommand help
#[test]
fn test_set_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("set")
        .status()
        .unwrap();
    assert!(status.success());
}

// test check unset subcommand help
#[test]
fn test_unset_help() {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("unset")
        .status()
        .unwrap();
    assert!(status.success());
}
