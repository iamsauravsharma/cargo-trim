use crate::{list_crate, ConfigFile, CrateDetail};
use colored::Colorize;
use std::{fs, path::Path};

// Store git dir folder information
pub(crate) struct GitDir<'a> {
    checkout_dir: &'a str,
    db_dir: &'a str,
    dry_run: bool,
}

impl<'a> GitDir<'a> {
    // create new GitDir
    pub(crate) fn new(checkout_dir: &'a Path, db_dir: &'a Path, dry_run: bool) -> Self {
        let checkout_dir = checkout_dir.to_str().unwrap();
        let db_dir = db_dir.to_str().unwrap();
        Self {
            checkout_dir,
            db_dir,
            dry_run,
        }
    }

    // remove crates
    pub(crate) fn remove_crate(&self, crate_name: &str) {
        if crate_name.contains("-HEAD") {
            remove_crate(Path::new(&self.db_dir), crate_name, self.dry_run);
        } else {
            remove_crate(Path::new(&self.checkout_dir), crate_name, self.dry_run);
        }
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
    pub(crate) fn remove_crate_list(&self, crate_detail: &CrateDetail, list: &[String]) -> f64 {
        let mut size_cleaned = 0.0;
        for crate_name in list {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find(crate_name, "GIT")
        }
        size_cleaned
    }

    // Remove all crate from git folder
    pub(crate) fn remove_all(
        &self,
        config_file: &ConfigFile,
        crate_name: &str,
        crate_detail: &CrateDetail,
    ) -> f64 {
        let crate_name = &crate_name.to_string();
        let mut size_cleaned = 0.0;
        let read_include = config_file.include();
        let read_exclude = config_file.exclude();
        let simple_name = crate_name
            .rsplitn(2, '-')
            .nth(1)
            .unwrap_or_default()
            .to_string();
        let env_include = list_crate::env_list("TRIM_INCLUDE");
        let env_exclude = list_crate::env_list("TRIM_EXCLUDE");

        if read_include.contains(crate_name)
            || read_include.contains(&simple_name)
            || env_include.contains(crate_name)
            || env_include.contains(&simple_name)
        {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
        if !read_exclude.contains(crate_name)
            && !read_exclude.contains(&simple_name)
            && !env_exclude.contains(crate_name)
            && !env_exclude.contains(&simple_name)
        {
            self.remove_crate(crate_name);
            size_cleaned += crate_detail.find_size_git_all(crate_name);
        }
        size_cleaned
    }
}

// preform remove operation
fn remove_crate(location: &Path, crate_name: &str, dry_run: bool) {
    for entry in fs::read_dir(location).expect("failed to read directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = crate_name.rsplitn(2, '-').collect::<Vec<&str>>();
        let crate_name = name[1];
        let rev_sha = name[0];
        if path.to_str().unwrap().contains(crate_name) {
            if rev_sha.contains("HEAD") {
                if dry_run {
                    println!("{} {} {:?}", "Dry run:".yellow(), "removed".red(), path);
                } else {
                    fs::remove_dir_all(&path).expect("failed to remove all directory");
                }
            } else {
                for rev in fs::read_dir(path).expect("failed to read git checkout directory") {
                    let entry = rev.unwrap();
                    let path = entry.path();
                    let file_name = path
                        .file_name()
                        .expect("path is terminating with ..")
                        .to_str()
                        .unwrap();
                    if file_name == rev_sha {
                        if dry_run {
                            println!("{} {} {:?}", "Dry run:".yellow(), "removed".red(), path);
                        } else {
                            fs::remove_dir_all(&path)
                                .expect("failed to remove all directory from Path");
                        }
                    }
                }
            }
        }
    }
}
