use std::collections::HashSet;

use owo_colors::OwoColorize;

use crate::crate_detail::CrateMetaData;
use crate::utils::convert_pretty;

/// show title
pub(super) fn show_title(title: &str, first_width: usize, second_width: usize, dash_len: usize) {
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|",
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
    dash_len: usize,
) {
    if data.is_empty() {
        println!(
            "|{:^first_width$}|{:^second_width$}|",
            "NONE".red(),
            convert_pretty(0).red(),
        );
    }
    print_dash(dash_len);
    println!(
        "|{:^first_width$}|{:^second_width$}|",
        format!("Total no of crates:- {}", data.len()).blue(),
        convert_pretty(size).blue(),
    );
    print_dash(dash_len);
}

/// print dash
pub(super) fn print_dash(len: usize) {
    println!("{}", "-".repeat(len));
}

/// top crates help to list out top n crates
pub(super) fn show_top_number_crates(
    crates: &HashSet<CrateMetaData>,
    crate_type: &str,
    number: usize,
) {
    // sort crates by size
    let mut crates = crates.iter().collect::<Vec<_>>();
    crates.sort_by_key(|a| std::cmp::Reverse(a.size()));
    let top_number = std::cmp::min(crates.len(), number);
    let title = format!("Top {top_number} {crate_type}");
    let mut listed_crates = Vec::new();
    for &crate_metadata in crates.iter().take(top_number) {
        listed_crates.push(crate_metadata.clone());
    }
    crate_list_type(&listed_crates[..top_number], &title);
}

// list certain crate type to terminal
pub(super) fn crate_list_type(crate_metadata_list: &[CrateMetaData], title: &str) {
    let first_width = 44;
    let second_width = 16;
    let dash_len = first_width + second_width + 3;
    show_title(title, first_width, second_width, dash_len);

    let mut total_size = 0;
    for crate_metadata in crate_metadata_list {
        let size = crate_metadata.size();
        total_size += size;
        if let Some(version) = crate_metadata.version() {
            println!(
                "|{:^first_width$}|{:^second_width$}|",
                format!("{}-{version}", crate_metadata.name()),
                convert_pretty(size)
            );
        } else {
            println!(
                "|{:^first_width$}|{:^second_width$}|",
                format!("{}", crate_metadata.name()),
                convert_pretty(size)
            );
        }
    }
    show_total_count(
        crate_metadata_list,
        total_size,
        first_width,
        second_width,
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
