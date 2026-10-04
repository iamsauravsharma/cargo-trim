use std::collections::HashSet;
use std::io::Write as _;

use anyhow::{Context as _, Result};
use owo_colors::OwoColorize as _;

use crate::crate_detail::{CrateDetail, CrateMetaData};
use crate::utils::convert_pretty;

pub(super) const OLD_ORPHAN_CLEAN_WARNING: &str =
    "WARNING: You have not initialized any directory as rust project directory. This command will \
     clean all old crates even if they are not orphan crates. Run command 'cargo trim set -d \
     <directory>' to set rust project directory, use 'cargo trim set -d .' for current directory";

pub(super) const ORPHAN_CLEAN_WARNING: &str =
    "WARNING: You have not initialized any directory as rust project directory. This command will \
     clean all crates since all crates are classified as orphan crate. Run command 'cargo trim \
     set -d <directory>' to set rust project directory, use 'cargo trim set -d .' for current \
     directory";

/// check if crates classified as orphan can be cleaned.
pub(super) fn confirm_orphan_clean(
    directory_is_empty: bool,
    warning_text: &str,
    dry_run: bool,
) -> Result<bool> {
    if !directory_is_empty {
        return Ok(true);
    }
    if dry_run {
        println!("{}", warning_text.yellow());
        return Ok(true);
    }
    confirm_continue(warning_text)
}

/// print provided warning text and ask user to confirm if they want to continue
/// If user enters "y" or "yes" (case insensitive), returns true, otherwise
/// returns false
///
/// # Errors
/// Returns an error if there is an issue flushing the output stream or reading
/// user input
pub(super) fn confirm_continue(warning_text: &str) -> Result<bool> {
    println!("{}", warning_text.yellow());
    print!("Do you want to continue? (y/N) ");
    std::io::stdout()
        .flush()
        .context("failed to flush output stream")?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("error: unable to read user input")?;
    let trimmed_input = input.trim().to_ascii_lowercase();
    Ok(["y", "yes"].contains(&trimmed_input.as_str()))
}

/// width of the location column based on the longest source name
/// Minimum width is 9 + 2 (for padding)
pub(super) fn source_name_max_width(crate_detail: &CrateDetail) -> usize {
    let max_length = crate_detail
        .source_infos()
        .keys()
        .map(String::len)
        .max()
        .unwrap_or_default();
    std::cmp::max(max_length, 9) + 2
}

/// width of the crate column based on the longest crate name with version
/// Minimum width is 30 + 2 (for padding)
pub(super) fn crate_name_max_width<'a, I>(crates: I) -> usize
where
    I: IntoIterator<Item = &'a CrateMetaData>,
{
    // width = crate name length + version length + 1 (for the hyphen) (if
    // version exists) width = crate name length (if version does not exist)
    let crate_name_max_width = crates
        .into_iter()
        .map(|cm| {
            cm.version().map_or(cm.name().len(), |version| {
                cm.name().len() + version.to_string().len() + 1
            })
        })
        .max()
        .unwrap_or_default();
    std::cmp::max(crate_name_max_width, 30) + 2
}

/// show title
pub(super) fn show_title(
    title: &str,
    first_width: usize,
    second_width: usize,
    third_width: usize,
    dash_len: usize,
) {
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|{:^third_width$}|",
        "LOCATION",
        title.bold(),
        "SIZE".bold(),
    );
    print_dash(dash_len);
}

/// show total count using data and size
pub(super) fn show_total_count(
    data: &[CrateMetaData],
    size: u64,
    first_width: usize,
    second_width: usize,
    third_width: usize,
    dash_len: usize,
) {
    if data.is_empty() {
        println!(
            "|{:^first_width$}|{:^second_width$}|{:^third_width$}|",
            "----",
            "NONE".red(),
            convert_pretty(0).red(),
        );
    }
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|{:^third_width$}|",
        "----",
        format!("Total no of crates:- {}", data.len()).blue(),
        convert_pretty(size).blue(),
    );
    print_dash(dash_len);
}

/// print dash
pub(super) fn print_dash(len: usize) {
    println!("{}", "-".repeat(len));
}

/// top crates help to List top n crates
pub(super) fn show_top_number_crates(
    crates: &HashSet<CrateMetaData>,
    crate_type: &str,
    first_width: usize,
    number: usize,
) {
    // sort crates by size and keep only the largest ones
    let mut top_crates = crates.iter().cloned().collect::<Vec<_>>();
    top_crates.sort_by_key(|a| std::cmp::Reverse(a.size()));
    top_crates.truncate(number);
    let title = format!("Top {} {crate_type}", top_crates.len());
    let second_width = crate_name_max_width(&top_crates);
    crate_list_type(&top_crates, first_width, second_width, &title);
}

// list certain crate type to terminal
pub(super) fn crate_list_type(
    crate_metadata_list: &[CrateMetaData],
    first_width: usize,
    second_width: usize,
    title: &str,
) {
    let third_width = 12;
    let dash_len = first_width + second_width + third_width + 4;
    show_title(title, first_width, second_width, third_width, dash_len);

    let mut total_size = 0;
    for crate_metadata in crate_metadata_list {
        let size = crate_metadata.size();
        total_size += size;
        if let Some(version) = crate_metadata.version() {
            println!(
                "|{:^first_width$}|{:^second_width$}|{:^third_width$}|",
                crate_metadata
                    .source()
                    .as_ref()
                    .map_or("N/A".to_string(), ToString::to_string),
                format!("{}-{version}", crate_metadata.name()),
                convert_pretty(size)
            );
        } else {
            println!(
                "|{:^first_width$}|{:^second_width$}|{:^third_width$}|",
                crate_metadata
                    .source()
                    .as_ref()
                    .map_or("N/A".to_string(), ToString::to_string),
                crate_metadata.name(),
                convert_pretty(size)
            );
        }
    }
    show_total_count(
        crate_metadata_list,
        total_size,
        first_width,
        second_width,
        third_width,
        dash_len,
    );
}

fn query_param_widths() -> (usize, usize) {
    (50, 10)
}

/// Get full width of query length
pub(super) fn query_full_width() -> usize {
    let (a, b) = query_param_widths();
    a + b + 1
}

/// Print query first and second params
pub(super) fn query_print(first_param: &str, second_param: &str) {
    let (first_path_width, second_path_width) = query_param_widths();
    println!("{first_param:first_path_width$} {second_param:>second_path_width$}");
}
