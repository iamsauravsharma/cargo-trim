use pretty_assertions::assert_eq;
use std::{io::Read, process::Command};

// test check trim subcommand help
#[test]
fn test_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/help.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

// test check list subcommand help
#[test]
fn test_list_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("list")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/list.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

// test check remove subcommand help
#[test]
fn test_remove_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("remove")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/remove.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

// test check config subcommand help
#[test]
fn test_config_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("config")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/config.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

// test check git subcommand help
#[test]
fn test_git_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("git")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/git.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

// test check registry subcommand help
#[test]
fn test_registry_help() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("trim")
        .arg("help")
        .arg("registry")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/registry.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}
