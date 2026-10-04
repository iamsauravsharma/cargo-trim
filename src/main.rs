mod cargo_config;
mod command;
mod config_file;
mod dir_path;
mod filter;
mod installed;
mod list_crate;
mod lock_file;
mod remove;
mod utils;

use std::env;

use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    let args = env::args();
    let mut command_args = Vec::new();
    for (pos, param) in args.enumerate() {
        if pos == 1 && param == "trim" {
            continue;
        }
        command_args.push(param);
    }

    let command = command::Command::parse_from(command_args);
    command.run()?;
    Ok(())
}
