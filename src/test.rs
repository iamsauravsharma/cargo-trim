use std::{io::Read, process::Command};
#[test]
fn test_help() {
    if !cfg!(target_os = "windows") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cargo run -- -h")
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8(output.stdout).unwrap();
        let mut buffer = String::new();
        let mut file = std::fs::File::open("tests/command_output/help.txt").unwrap();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(output, buffer);
    }
}

#[test]
fn test_list_help() {
    if !cfg!(target_os = "windows") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cargo run -- help list")
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8(output.stdout).unwrap();
        let mut buffer = String::new();
        let mut file = std::fs::File::open("tests/command_output/list.txt").unwrap();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(output, buffer);
    }
}

#[test]
fn test_remove_help() {
    if !cfg!(target_os = "windows") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cargo run -- help remove")
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8(output.stdout).unwrap();
        let mut buffer = String::new();
        let mut file = std::fs::File::open("tests/command_output/remove.txt").unwrap();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(output, buffer);
    }
}

#[test]
fn test_query_help() {
    if !cfg!(target_os = "windows") {
        let output = Command::new("sh")
            .arg("-c")
            .arg("cargo run -- help query")
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8(output.stdout).unwrap();
        let mut buffer = String::new();
        let mut file = std::fs::File::open("tests/command_output/query.txt").unwrap();
        file.read_to_string(&mut buffer).unwrap();
        assert_eq!(output, buffer);
    }
}
