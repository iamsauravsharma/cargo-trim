use std::{
    fs,
    io::prelude::*,
    path::{Path, PathBuf},
};

mod create_app;

fn main() {
    let mut config_dir = dirs::config_dir().unwrap();
    let mut home_dir = dirs::home_dir().unwrap();
    home_dir.push(".cargo");
    home_dir.push("registry");
    let cargo_cache_config = format!(
        "{}/{}",
        config_dir.to_str().unwrap(),
        "cargo_cache_config.txt"
    );
    let mut file = fs::File::open(&cargo_cache_config)
        .unwrap_or_else(|_| fs::File::create(&cargo_cache_config).unwrap());
    config_dir.push("cargo_cache_config.txt");
    let app = create_app::app();

    write_to_file("set directory", &mut file, &app, &config_dir);
    write_to_file("exclude-conf", &mut file, &app, &config_dir);
    write_to_file("include-conf", &mut file, &app, &config_dir);

    let mut cache_dir = home_dir.clone();
    cache_dir.push("cache");
    let mut src_dir = home_dir.clone();
    src_dir.push("src");
    let git_cache_dir = open_github_folder(&cache_dir).unwrap();
    let git_src_dir = open_github_folder(&src_dir).unwrap();
    let installed_crate = list_crate(Path::new(&git_src_dir));

    if app.is_present("list") {
        for list in installed_crate.iter() {
            println!("{}", list);
        }
    }

    if app.is_present("remove") {
        let value = app.value_of("remove").unwrap();
        remove_value(Path::new(&git_cache_dir), value);
        remove_value(Path::new(&git_src_dir), value);
    }

    if app.is_present("all") {
        let home_dir = home_dir.clone();
        fs::remove_dir_all(home_dir).unwrap();
    }
}

fn open_github_folder(path: &Path) -> Option<String> {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();
        if path.contains("github.com") {
            return Some(path.to_string());
        }
    }
    None
}

fn list_crate(src_dir: &Path) -> Vec<String> {
    let mut list = Vec::new();
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap().path();
        let path = entry.as_path();
        let file_name = path.file_name().unwrap();
        let crate_name = file_name.to_str().unwrap().to_string();
        list.push(crate_name)
    }
    list
}

fn write_to_file(name: &str, file: &mut fs::File, app: &clap::ArgMatches, config_dir: &PathBuf) {
    if app.is_present(name) {
        let value = app.value_of(name).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        if !buffer.is_empty() {
            buffer.push_str("\n");
        }
        let title = match name {
            "set directory" => "directory",
            "exclude-conf" => "exclude",
            "include-conf" => "include",
            _ => "",
        };
        buffer.push_str(format!("{}:{}", title, value).as_str());
        fs::write(config_dir, buffer.as_bytes()).unwrap();
    }
}

fn remove_value(path: &Path, value: &str) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.to_str().unwrap().contains(value) {
            if path.is_file() {
                fs::remove_file(path).unwrap();
            } else if path.is_dir() {
                fs::remove_dir_all(path).unwrap();
            }
        } else if path.is_dir() {
            remove_value(&path, value);
        }
    }
}
