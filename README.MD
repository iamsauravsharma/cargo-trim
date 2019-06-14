# CARGO-TRIM

**Status:**

| Travis Build Status | Dependency Status |
| :---: | :---: | 
| [![Travis Build Status][build_badge]][build_link] | [![Dependency status][deps_badge]][deps_link] |

**Project Info:**

| License | LoC | Crates Version |
| :---: | :---: | :---: |
| [![License: MIT][license_badge]][license_link] | [![LoC][loc_badge]][loc_link] | [![Crate][cratesio_badge]][cratesio_link] |

**Rust version support:**

| Stable | Beta | Nightly |
| :---: | :---: | :---: |
| [![Stable support][stable_supported_badge]][git_link] | [![Beta support][beta_supported_badge]][git_link] | [![Nightly support][nightly_supported_badge]][git_link] |

**OS support:**

| Linux | OSX | Windows |
| :---: | :---: | :---: |
| [![Linux support][linux_supported_badge]][git_link] | [![OSX support][osx_supported_badge]][git_link] | [![Windows support][windows_supported_badge]][git_link] |

Binary application for cleaning __\$CARGO_HOME/registry__  & __\$CARGO_HOME/git__ folder support orphan cleaning and old cleaning
Default \$CARGO_HOME is set as \$HOME/.cargo

### Install
Initially you should have rustc and cargo installed for a detail information of how to install rust-lang visit https://rust-lang.org

For installation of cargo-trim:-
You can install from crates.io registry
```
    cargo install cargo-trim
```
__OR__

For bleeding edge latest development version:
```
    cargo install --git https://github.com/iamsauravsharma/cargo-trim
```

By default cargo-trim print out colored output if you want to use a non colored output you can enable out non-colored-output using feature flag
```
    cargo install cargo-trim --features non-colored-output
```

### Command output
All cargo-trim can be run using ```cargo trim <command>```
```cargo trim --help``` will produce out following output which list out all of available output for cargo-trim
```
cargo-trim 0.2.2
Saurav Sharma <appdroiddeveloper@gmail.com>
Binary application to cleanup $CARGO_HOME cache

USAGE:
    cargo trim [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -a, --all             Clean up all .cargo/registry & .cargo/git follow config file data
    -c, --clear           Clear config data
    -f, --force           Force clear cache without reading conf file
    -h, --help            Prints help information
    -l, --light           Light cleanup repos by removing git checkout and registry source but stores git db and
                          registry archive for future compilation
    -o, --old-clean       Clean old cache crates
    -x, --orphan-clean    Clean orphan cache crates
    -q, --query           Return size of .cargo/cache folders
    -u, --update          Update Cargo.lock file present inside config directory folder path
    -V, --version         Prints version information

OPTIONS:
    -e, --exclude <Crate>...              add listed crates to default conf file exclude list
    -g, --gc <git compress>               Git compress to reduce size of .cargo [possible values: all, index, git, git-
                                          checkout, git-db]
    -i, --include <Crate>...              add listed crates to default conf file include list
    -r, --remove <Crate>...               Remove listed crates
    -s, --set-directory <Directory>...    Set directory of Rust project
    -t, --top <number>                    Show certain number of top crates which have highest size
    -w, --wipe <Folder>                   Wipe folder [possible values: git, checkouts, db, registry, cache, index, src]

SUBCOMMANDS:
    clear       Clear current working directory from cargo cache config [alias: "clr"]
    config      Query config file data [alias: "c"]
    git         Perform operation only to git related cache file [alias: "g"]
    help        Prints this message or the help of the given subcommand(s)
    init        Initialize current working directory as cargo trim directory [alias: "in"]
    list        List out crates [alias: "l"]
    registry    Perform operation only to registry related cache file [alias: "reg"]
    remove      Remove values from config file [alias: "rm"]
```

### Config file location
cargo-trim store its config file to config directory of OS and name config file as cago_trim_config.json.
In file cargo-trim stores out information about directory of rust projects and exclude and include crates list.

[git_link]: https://github.com/iamsauravsharma/cargo-trim

[build_badge]: https://img.shields.io/travis/com/iamsauravsharma/cargo-trim.svg?logo=travis
[build_link]: https://travis-ci.com/iamsauravsharma/cargo-trim

[deps_badge]: https://deps.rs/repo/github/iamsauravsharma/cargo-trim/status.svg
[deps_link]: https://deps.rs/repo/github/iamsauravsharma/cargo-trim

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg
[license_link]: LICENSE

[loc_badge]: https://tokei.rs/b1/github/iamsauravsharma/cargo-trim
[loc_link]: https://github.com/iamsauravsharma/cargo-trim

[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg
[cratesio_link]: https://crates.io/crates/cargo-trim

[stable_supported_badge]: https://img.shields.io/badge/stable-supported-brightgreen.svg?logo=rust
[beta_supported_badge]: https://img.shields.io/badge/beta-supported-brightgreen.svg?logo=rust
[nightly_supported_badge]: https://img.shields.io/badge/nightly-supported-brightgreen.svg?logo=rust
[stable_not_supported_badge]: https://img.shields.io/badge/stable-not%20supported-red.svg?logo=rust
[beta_not_supported_badge]: https://img.shields.io/badge/beta-not%20supported-red.svg?logo=rust
[nightly_not_supported_badge]: https://img.shields.io/badge/nightly-not%20supported-red.svg?logo=rust

[linux_supported_badge]: https://img.shields.io/badge/linux-supported-brightgreen.svg?logo=linux
[osx_supported_badge]: https://img.shields.io/badge/osx-supported-brightgreen.svg?logo=apple
[windows_supported_badge]: https://img.shields.io/badge/windows-supported-brightgreen.svg?logo=windows
[linux_not_supported_badge]: https://img.shields.io/badge/linux-not%20supported-red.svg?logo=linux
[osx_not_supported_badge]: https://img.shields.io/badge/osx-not%20supported-red.svg?logo=apple
[nightly_not_supported_badge]: https://img.shields.io/badge/windows-not%20supported-red.svg?logo=windows