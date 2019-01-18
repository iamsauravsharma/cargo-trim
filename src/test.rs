use std::{io::Read, process::Command};
#[test]
fn test_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim -h")
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

#[test]
fn test_list_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim help list")
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

#[test]
fn test_remove_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim help remove")
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

#[test]
fn test_query_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim help query")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();
    let mut buffer = String::new();
    let mut file = std::fs::File::open("tests/command_output/query.txt").unwrap();
    file.read_to_string(&mut buffer).unwrap();
    assert_eq!(
        output.replace("\r", "").replace("\n", ""),
        buffer.replace("\r", "").replace("\n", "")
    );
}

#[test]
fn test_git_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim help git")
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

#[test]
fn test_registry_help() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run -- trim help registry")
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
