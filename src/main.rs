extern crate clap;

use clap::{App, Arg,ArgMatches};

fn main() {
    let app = App::new("Cargo Cache")
        .version("0.1.0")
        .author("Saurav Sharma <appdroiddeveloper@gmail.com>")
        .about("Clean cache from .cargo/registry")
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Clean up all .cargo/registry")
                .takes_value(false),
        )
        .get_matches();
        matches(app);
}

fn matches(app : ArgMatches){
    let full_clean = app.is_present("all");
    println!("{:?}",full_clean);
}
