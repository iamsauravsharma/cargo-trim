extern crate clap;

use clap::{App, Arg, ArgMatches};

fn main() {
    let app = App::new("Cargo Cache")
        .version("0.1.0")
        .author("Saurav Sharma <appdroiddeveloper@gmail.com>")
        .about("Clean cache from .cargo/registry")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Clean up all .cargo/registry"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Set a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("set directory")
                .short("s")
                .long("set-directory")
                .value_name("Directory")
                .help("Set directory of Rust project")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("orphan clean")
                .short("O")
                .long("orphan-clean")
                .help("Clean orphan cache crates"),
        )
        .arg(
            Arg::with_name("Old clean")
                .short("o")
                .long("old-clean")
                .help("Clean old cache crates"),
        )
        .arg(
            Arg::with_name("exclude")
                .short("e")
                .long("exclude")
                .help("Exculde listed crates")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .arg(
            Arg::with_name("include")
                .short("i")
                .long("include")
                .help("Include listed crates")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .get_matches();
    matches(app);
}

fn matches(app: ArgMatches) {
    let _full_clean = app.is_present("all");
    let _config_file = app.value_of("config").unwrap_or(".cargo_cache.conf");
}
