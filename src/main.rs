#![warn(unreachable_pub, anonymous_parameters, bare_trait_objects)]
#![deny(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines, clippy::struct_excessive_bools)]

mod command;
mod config_file;
mod crate_detail;
mod dir_path;
mod git_dir;
mod list_crate;
mod registry_dir;
mod utils;

use std::env;

use anyhow::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    let args = env::args();
    let mut command_args = Vec::new();
    for (pos, param) in args.enumerate() {
        if pos == 1 && param == "trim" {
            continue;
        }
        command_args.push(param);
    }

    let command = command::Command::from_iter(command_args);
    command.run()?;
    Ok(())
}
