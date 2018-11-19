use clap::{App, Arg, ArgMatches};

pub(super) fn app() -> ArgMatches<'static> {
    App::new("Cargo Cleaner")
        .version("0.1.0")
        .author("Saurav Sharma <appdroiddeveloper@gmail.com>")
        .about("Clean cache from .cargo/registry")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Clean up all .cargo/registry follow config file data"),
        )
        .arg(
            Arg::with_name("clear config")
                .short("c")
                .long("clear")
                .help("Clear config data"),
        )
        .arg(
            Arg::with_name("exclude")
                .short("e")
                .long("exclude")
                .help("Exclude listed crates")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .arg(
            Arg::with_name("exclude-conf")
                .short("E")
                .long("exclude-conf")
                .help("add listed crates to default conf file exclude list")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .arg(
            Arg::with_name("force remove")
                .short("f")
                .long("force")
                .help("Force clear cache without reading conf file"),
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
        .arg(
            Arg::with_name("include-conf")
                .short("I")
                .long("include-conf")
                .help("add listed crates to default conf file include list")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("List installed crate"),
        )
        .arg(
            Arg::with_name("old clean")
                .short("o")
                .long("old-clean")
                .help("Clean old cache crates"),
        )
        .arg(
            Arg::with_name("orphan clean")
                .short("d")
                .long("orphan-clean")
                .help("Clean orphan cache crates"),
        )
        .arg(
            Arg::with_name("set directory")
                .short("s")
                .multiple(true)
                .long("set-directory")
                .value_name("Directory")
                .help("Set directory of Rust project")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("query size")
                .short("q")
                .long("size")
                .help("Return size of .cargo/registry"),
        )
        .arg(
            Arg::with_name("remove")
                .short("r")
                .long("remove")
                .help("Remove listed crates")
                .multiple(true)
                .takes_value(true)
                .value_name("Crate"),
        )
        .get_matches()
}
