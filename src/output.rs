use std::sync::atomic::{AtomicBool, Ordering};

use crate::color::Paint as _;

static DRY_RUN: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Set output from the dry run, quiet and verbose flags, quiet and verbose
/// cannot both be set
pub(crate) fn set(dry_run: bool, quiet: bool, verbose: bool) {
    DRY_RUN.store(dry_run, Ordering::Relaxed);
    QUIET.store(quiet, Ordering::Relaxed);
    VERBOSE.store(verbose, Ordering::Relaxed);
}

/// `Dry run: ` marker put before every [`info`] and [`outcome`] line under dry
/// run, so a dry run stays recognisable even when quiet output hides progress
pub(crate) fn dry_run_prefix() -> String {
    if is_dry_run() {
        format!("{} ", "Dry run:".yellow())
    } else {
        String::new()
    }
}

/// Whether nothing should be changed, only reported
pub(crate) fn is_dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed)
}

/// Whether quiet output is chosen
pub(crate) fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

/// Whether verbose output is chosen
pub(crate) fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Print progress line to standard output unless quiet output is chosen,
/// marked under dry run
macro_rules! info {
    ($($arg:tt)*) => {
        if !$crate::output::is_quiet() {
            $crate::output::outcome!($($arg)*);
        }
    };
}

/// Print result line to standard output even with quiet output, marked under
/// dry run
macro_rules! outcome {
    ($($arg:tt)*) => {
        println!("{}{}", $crate::output::dry_run_prefix(), format_args!($($arg)*))
    };
}

/// Print detail line to standard error when verbose output is chosen,
/// standard output stays the same with or without it
macro_rules! verbose {
    ($($arg:tt)*) => {
        if $crate::output::is_verbose() {
            eprintln!(
                "{} {}",
                $crate::color::PaintErr::dimmed(&"[verbose]"),
                format_args!($($arg)*)
            );
        }
    };
}

pub(crate) use info;
pub(crate) use outcome;
pub(crate) use verbose;
