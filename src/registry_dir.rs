use crate::{
    list_crate,
    utils::{clear_version_value, delete_folder},
    ConfigFile, CrateDetail,
};
use colored::Colorize;
use std::{fs, path::Path};

// Stores .cargo/registry cache & src information
pub(crate) struct RegistryDir<'a> {
    cache_dir: &'a str,
    src_dir: &'a str,
    index_cache_dir: Vec<String>,
    installed_crate: Vec<String>,
    dry_run: bool,
}

impl<'a> RegistryDir<'a> {
    // Create new RegistryDir
    pub(crate) fn new(
        cache_dir: &'a Path,
        src_dir: &'a Path,
        index_dir: &Path,
        installed_crate: &[String],
        dry_run: bool,
    ) -> Self {
        let cache_dir = cache_dir.to_str().unwrap();
        let src_dir = src_dir.to_str().unwrap();
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
        let mut is_success;
        is_success = remove_crate(Path::new(&self.cache_dir), crate_name, self.dry_run).is_ok();
        is_success =
            remove_crate(Path::new(&self.src_dir), crate_name, self.dry_run).is_ok() && is_success;
        let split_value = clear_version_value(crate_name);
        let name = split_value.0;
        let index_cache = self.index_cache_dir.to_owned();
        index_cache.iter().for_each(|index_cache_dir| {
            let same_name_list: Vec<&String> = self
                .installed_crate
                .iter()
                .filter(|&x| x.contains(&name))
                .collect();
            if same_name_list.len() == 1 {
                is_success =
                    remove_index_cache(Path::new(&index_cache_dir), crate_name, self.dry_run)
                        .is_ok()
                        && is_success;
            }
            is_success = remove_empty_index_cache_dir(Path::new(&index_cache_dir), self.dry_run)
                .is_ok()
                && is_success;
            self.installed_crate.retain(|x| x != crate_name);
        });
        if self.dry_run {
            println!(
                "{} {} {:?}",
                "Dry run:".color("yellow"),
                "Removed".color("red"),
                crate_name
            );
        } else if is_success {
            println!("{} {:?}", "Removed".color("red"), crate_name);
        } else {
            println!(
                "Partially failed to remove some directory and file of {:?}",
                crate_name
            )
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
        let simple_name = clear_version_value(crate_name).0;
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
fn remove_crate(path: &Path, value: &str, dry_run: bool) -> std::io::Result<()> {
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.to_str().unwrap().contains(value) {
                    delete_folder(&path, dry_run)?;
                }
            }
        }
    }
    Ok(())
}

fn remove_index_cache(path: &Path, crate_name: &str, dry_run: bool) -> std::io::Result<()> {
    let mut remove_file_location = path.to_path_buf();
    let split_value = clear_version_value(crate_name);
    let name = split_value.0;
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
    delete_folder(&remove_file_location, dry_run)?;
    Ok(())
}

fn remove_empty_index_cache_dir(path: &Path, dry_run: bool) -> std::io::Result<()> {
    if path
        .read_dir()
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
    {
        delete_folder(&path, dry_run)?
    } else {
        for entry in path.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                remove_empty_index_cache_dir(path.as_path(), dry_run)?;
            }
        }
    }
    Ok(())
}
