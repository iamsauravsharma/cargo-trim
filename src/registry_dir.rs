use crate::{list_crate, ConfigFile, CrateDetail};
use colored::*;
use std::{fs, path::Path};

// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir {
    cache_dir: String,
    src_dir: String,
    index_cache_dir: Vec<String>,
    installed_crate: Vec<String>,
    dry_run: bool,
}

impl RegistryDir {
    // Create new RegistryDir
    pub(crate) fn new(
        cache_dir: &Path,
        src_dir: &Path,
        index_dir: &Path,
        installed_crate: &[String],
        dry_run: bool,
    ) -> Self {
        let cache_dir = cache_dir.to_str().unwrap().to_string();
        let src_dir = src_dir.to_str().unwrap().to_string();
        let mut index_cache_dir = Vec::new();
        for entry in fs::read_dir(index_dir).expect("failed to read index directory") {
            let entry = entry.unwrap().path();
            let registry_dir = entry.as_path();
            for folder in fs::read_dir(registry_dir).expect("failed to read registry directory") {
                let folder = folder.unwrap().path();
                let folder_name = folder
                    .file_name()
                    .expect("failed to get file name form registry sub directory");
                if folder_name == ".cache" {
                    index_cache_dir.push(folder.to_str().unwrap().to_string());
                }
            }
        }

        Self {
            cache_dir,
            src_dir,
            index_cache_dir,
            installed_crate: installed_crate.to_owned(),
            dry_run,
        }
    }

    // Remove crate from src & cache directory
    pub(crate) fn remove_crate(&mut self, crate_name: &str) {
        remove_crate(Path::new(&self.cache_dir), crate_name, self.dry_run);
        remove_crate(Path::new(&self.src_dir), crate_name, self.dry_run);
        let splitted_value: Vec<&str> = crate_name.rsplitn(2, '-').collect();
        let name = splitted_value[1];
        let index_cache = self.index_cache_dir.to_owned();
        index_cache.iter().for_each(|index_cache_dir| {
            let same_name_list: Vec<&String> = self
                .installed_crate
                .iter()
                .filter(|x| x.contains(name))
                .collect();
            if same_name_list.len() == 1 {
                remove_index_cache(Path::new(&index_cache_dir), crate_name);
            }
            remove_empty_index_cache_dir(Path::new(&index_cache_dir));
            self.installed_crate.retain(|x| x != crate_name);
        });
        if self.dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".yellow(),
                "removed".red(),
                crate_name
            );
        } else {
            println!("{} {:?}", "Removed".red(), crate_name);
        }
    }

    // Remove list of crates
    pub(crate) fn remove_crate_list(&mut self, crate_detail: &CrateDetail, list: &[String]) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find(crate_name, "REGISTRY")
        }
        size_cleaned
    }

    // Remove all crates from registry folder
    pub(crate) fn remove_all(
        &mut self,
        config_file: &ConfigFile,
        crate_name: &str,
        crate_detail: &CrateDetail,
    ) -> f64 {
        let crate_name = &crate_name.to_string();
        let mut sized_cleaned = 0.0;

        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        let crate_split = crate_name.rsplitn(2, '-');
        let mut simple_name = String::new();
        for (i, val) in crate_split.enumerate() {
            if i == 1 {
                simple_name = val.to_string()
            }
        }
        let env_include = list_crate::env_list("TRIM_INCLUDE");
        let env_exclude = list_crate::env_list("TRIM_EXCLUDE");

        if read_include.contains(crate_name)
            || read_include.contains(&simple_name)
            || env_include.contains(crate_name)
            || env_include.contains(&simple_name)
        {
            self.remove_crate(crate_name);
            sized_cleaned += crate_detail.find_size_registry_all(crate_name);
        }
        if !read_exclude.contains(crate_name)
            && !read_exclude.contains(&simple_name)
            && !env_exclude.contains(crate_name)
            && !env_exclude.contains(&simple_name)
        {
            self.remove_crate(crate_name);
            sized_cleaned += crate_detail.find_size_registry_all(crate_name);
        }
        sized_cleaned
    }
}

// Remove crates which name is provided to delete
fn remove_crate(path: &Path, value: &str, dry_run: bool) {
    if path.exists() {
        for entry in fs::read_dir(path).expect("failed to read src or cache dir") {
            let entry = entry.unwrap();
            let path = entry.path();
            for entry in fs::read_dir(path).expect("failed to read crates path") {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.to_str().unwrap().contains(value) {
                    if path.is_file() {
                        if dry_run {
                            println!("{} {} {:?}", "Dry run:".yellow(), "removed".red(), path);
                        } else {
                            fs::remove_file(&path).expect("failed to remove file");
                        }
                    } else if path.is_dir() {
                        if dry_run {
                            println!("{} {} {:?}", "Dry run:".yellow(), "removed".red(), path);
                        } else {
                            fs::remove_dir_all(&path)
                                .expect("failed to remove all directory contents");
                        }
                    }
                }
            }
        }
    }
}

fn remove_index_cache(path: &Path, value: &str) {
    let mut remove_file_location = path.to_path_buf();
    let splitted_value: Vec<&str> = value.rsplitn(2, '-').collect();
    let name = splitted_value[1];
    match name.len() {
        1 => {
            remove_file_location.push("1");
            remove_file_location.push(name);
        }
        2 => {
            remove_file_location.push("2");
            remove_file_location.push(name);
        }
        3 => {
            remove_file_location.push("3");
            remove_file_location.push(&name[..1]);
            remove_file_location.push(name);
        }
        _ => {
            remove_file_location.push(&name[..2]);
            remove_file_location.push(&name[2..4]);
            remove_file_location.push(name);
        }
    };
    if remove_file_location.exists() && remove_file_location.is_file() {
        fs::remove_file(remove_file_location).expect("Failed to remove .cache file");
    }
}

fn remove_empty_index_cache_dir(path: &Path) {
    if path
        .read_dir()
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
    {
        fs::remove_dir(path).expect("Failed to remove empty index cache");
    } else {
        for entry in path.read_dir().expect("Failed to read .cache sub folder") {
            let path = entry.unwrap().path();
            if path.is_dir() {
                remove_empty_index_cache_dir(path.as_path())
            }
        }
    }
}
