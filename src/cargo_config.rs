use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::Deserialize;

/// Cargo configuration file structure
#[derive(Deserialize)]
struct CargoConfig {
    build: Option<BuildSection>,
}

#[derive(Deserialize)]
struct BuildSection {
    #[serde(rename = "target-dir")]
    target_dir: Option<String>,
}

/// Cache for the environment and every cargo config file already looked at, so
/// neither is read twice
#[derive(Default)]
pub(crate) struct TargetDirCache {
    env_value: OnceCell<Option<PathBuf>>,
    configured: HashMap<PathBuf, Option<PathBuf>>,
}

/// Resolve the target directory Cargo uses for a project, following the same
/// order Cargo does: environment variables, project-specific config files,
/// Cargo home config, and finally the default "target" directory in the
/// project.
pub(crate) fn target_dir(project_dir: &Path, cache: &mut TargetDirCache) -> PathBuf {
    let env_value = cache.env_value.get_or_init(get_target_dir_env);
    if let Some(value) = env_value {
        return resolve(project_dir, value);
    }
    for ancestor in project_dir.ancestors() {
        if let Some(target_dir) = cached_target_dir(&ancestor.join(".cargo"), cache) {
            return target_dir;
        }
    }
    if let Some(cargo_home) = env::var_os("CARGO_HOME")
        && let Some(target_dir) = cached_target_dir(Path::new(&cargo_home), cache)
    {
        return target_dir;
    }
    project_dir.join("target")
}

fn get_target_dir_env() -> Option<PathBuf> {
    for variable in ["CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"] {
        if let Some(value) = env::var_os(variable).filter(|value| !value.is_empty()) {
            return Some(PathBuf::from(value));
        }
    }
    None
}

fn cached_target_dir(config_dir: &Path, cache: &mut TargetDirCache) -> Option<PathBuf> {
    if let Some(cached) = cache.configured.get(config_dir) {
        return cached.clone();
    }
    // cargo resolves a relative value against the parent of the directory
    // holding the config file
    let base = config_dir.parent().unwrap_or(config_dir);
    let target_dir = ["config.toml", "config"]
        .into_iter()
        .find_map(|file_name| {
            let content = fs::read_to_string(config_dir.join(file_name)).ok()?;
            toml::from_str::<CargoConfig>(&content)
                .ok()?
                .build?
                .target_dir
        })
        .map(|value| resolve(base, Path::new(&value)));
    cache
        .configured
        .insert(config_dir.to_path_buf(), target_dir.clone());
    target_dir
}

fn resolve(base: &Path, value: &Path) -> PathBuf {
    if value.is_absolute() {
        value.to_path_buf()
    } else {
        base.join(value)
    }
}
