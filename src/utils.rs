use colored::Colorize;
use std::{fs, path::Path};

// delete folder with folder path provided
pub(crate) fn delete_folder(path: &Path, dry_run: bool) {
    if path.exists() {
        if path.is_file() {
            if dry_run {
                println!(
                    "{} {} {:?}",
                    "Dry run:".color("yellow"),
                    "removed".color("red"),
                    path
                );
            } else {
                fs::remove_file(&path).expect("failed to remove file");
            }
        } else if path.is_dir() {
            if dry_run {
                println!(
                    "{} {} {:?}",
                    "Dry run:".color("yellow"),
                    "removed".color("red"),
                    path
                );
            } else {
                fs::remove_dir_all(path).expect("failed to remove all directory content");
            }
        }
    }
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
