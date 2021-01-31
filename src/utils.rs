use colored::Colorize;
use std::{env, fs, path::Path};

// list all a env variables list in vector form
pub(crate) fn env_list(variable: &str) -> Vec<String> {
    let list = env::var(variable);
    let mut vec_list = Vec::new();
    if let Ok(name_list) = list {
        name_list
            .split_whitespace()
            .for_each(|name| vec_list.push(name.to_string()));
    }
    vec_list
}

// remove semver version part from crates full name
pub(crate) fn clear_version_value(full_name: &str) -> (String, String) {
    let version_split: Vec<&str> = full_name.split('-').collect();
    let mut version_start_position = version_split.len();
    // check a split part to check from where a semver start
    for (pos, split_part) in version_split.iter().enumerate() {
        if semver::Version::parse(split_part).is_ok() {
            version_start_position = pos;
            break;
        }
    }
    let (clear_name_vec, version_vec) = version_split.split_at(version_start_position);
    let clear_name = clear_name_vec.join("-");
    let version = version_vec.join("-");
    (clear_name, version)
}

// delete folder with folder path provided
pub(crate) fn delete_folder(path: &Path, dry_run: bool) -> std::io::Result<()> {
    if path.exists() {
        if path.is_file() {
            if dry_run {
                println!(
                    "{} {} {:?}",
                    "Dry run:".color("yellow"),
                    "Removed".color("red"),
                    path
                );
            } else {
                fs::remove_file(&path)?;
            }
        } else if path.is_dir() {
            if dry_run {
                println!(
                    "{} {} {:?}",
                    "Dry run:".color("yellow"),
                    "Removed".color("red"),
                    path
                );
            } else {
                fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}

//  get size of directory
pub(crate) fn get_size(path: &Path) -> std::io::Result<u64> {
    let mut total_size = 0;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry_path = entry?.path();
            if entry_path.is_dir() {
                total_size += get_size(&entry_path)?;
            } else {
                total_size += entry_path.metadata()?.len();
            }
        }
    } else {
        total_size += path.metadata()?.len();
    }
    Ok(total_size)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub(crate) fn convert_pretty(num: u64) -> String {
    if num == 0 {
        return "0 B".to_string();
    }
    let num = num as f64;
    let units = ["B", "kB", "MB", "GB", "TB"];
    let factor = (num.log10() / 3_f64).floor();
    let power_factor = if factor >= units.len() as f64 {
        (units.len() - 1) as f64
    } else {
        factor
    };
    let pretty_bytes = format!("{:.3}", num / 1000_f64.powf(power_factor))
        .parse::<f64>()
        .unwrap();
    let unit = units[power_factor as usize];
    format!("{} {}", pretty_bytes, unit)
}

#[cfg(test)]
mod test {
    use super::{clear_version_value, convert_pretty, env_list};

    #[test]
    fn test_env_list() {
        std::env::set_var("SAMPLE_ENV", "MULTIPLE LIST OF VALUE");
        assert_eq!(
            env_list("SAMPLE_ENV"),
            vec![
                "MULTIPLE".to_string(),
                "LIST".to_string(),
                "OF".to_string(),
                "VALUE".to_string()
            ]
        );
        assert!(env_list("RANDOM_ENV").is_empty());
    }

    #[test]
    fn test_clear_version_value() {
        assert_eq!(
            clear_version_value("sample_crate-0.12.0"),
            ("sample_crate".to_string(), "0.12.0".to_string())
        );
        assert_eq!(
            clear_version_value("another-crate-name-1.4.5"),
            ("another-crate-name".to_string(), "1.4.5".to_string())
        );
        assert_eq!(
            clear_version_value("crate-name-12-123-0.1.0"),
            ("crate-name-12-123".to_string(), "0.1.0".to_string())
        );
        assert_eq!(
            clear_version_value("complex_name-12.0.0-rc.1"),
            ("complex_name".to_string(), "12.0.0-rc.1".to_string())
        );
        assert_eq!(
            clear_version_value("build-number-2.3.4+was0-5"),
            ("build-number".to_string(), "2.3.4+was0-5".to_string())
        );
        assert_eq!(
            clear_version_value("complex_spec-0.12.0-rc.1+name0.4.6"),
            (
                "complex_spec".to_string(),
                "0.12.0-rc.1+name0.4.6".to_string()
            )
        );
    }

    #[test]
    fn test_convert_pretty() {
        assert_eq!(convert_pretty(0), "0 B".to_string());
        assert_eq!(convert_pretty(12), "12 B".to_string());
        assert_eq!(convert_pretty(1234), "1.234 kB".to_string());
        assert_eq!(convert_pretty(23908), "23.908 kB".to_string());
        assert_eq!(convert_pretty(874940334), "874.94 MB".to_string());
        assert_eq!(convert_pretty(8849909404), "8.85 GB".to_string());
        assert_eq!(convert_pretty(3417849409404), "3.418 TB".to_string());
        assert_eq!(
            convert_pretty(93453982182159417),
            "93453.982 TB".to_string()
        );
    }
}
