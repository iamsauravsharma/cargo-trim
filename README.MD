# CARGO-TRIM

**Status:**
[![Travis Build Status][build_badge]][build_link]
[![Dependency status][deps_badge]][deps_link]

**Project Info:**
[![License: MIT][license_badge]][license_link]
[![LoC][loc_badge]][loc_link]

**Rust version support**
[![Stable support][stable_supported_badge]][git_link]
[![Beta support][beta_supported_badge]][git_link]
[![Nightly support][nightly_supported_badge]][git_link]


**OS support**
[![Linux support][linux_supported_badge]][git_link]
[![OSX support][osx_supported_badge]][git_link]
[![Windows support][windows_supported_badge]][git_link]

(WIP) Binary application for cleaning $HOME/.cargo/registry file

### TODO
- [ ] Proper handling of error
- [ ] Support git import crates
- [ ] Publish crate to [crates.io][cratesio_link]

### Install
Initially you should have cargo installed
```
    cargo install --git https://github.com/iamsauravsharma/cargo-trim
```

### Command output
```cargo trim -h``` will produce out following output
```
cargo-trim v 0.1.0
Saurav Sharma <appdroiddeveloper@gmail.com>
Clean cache from .cargo/registry

USAGE:
    cargo trim [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -a, --all             Clean up all .cargo/registry follow config file data
    -c, --clear           Clear config data
    -f, --force           Force clear cache without reading conf file
    -h, --help            Prints help information
    -o, --old-clean       Clean old cache crates
    -x, --orphan-clean    Clean orphan cache crates
    -s, --size            Return size of .cargo/registry
    -V, --version         Prints version information

OPTIONS:
    -e, --exclude <Crate>...          Exclude listed crates
    -E, --exclude-conf <Crate>...     add listed crates to default conf file exclude list
    -i, --include <Crate>...          Include listed crates
    -I, --include-conf <Crate>...     add listed crates to default conf file include list
    -r, --remove <Crate>...           Remove listed crates
    -d, --directory <Directory>...    Set directory of Rust project
    -w, --wipe <Folder>               Wipe folder expected value : registry, cache, index, src

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    list      List out all of installed crates
    query     Query config file data
    remove    Remove values from config file
```

[git_link]: https://github.com/iamsauravsharma/cargo-trim

[build_badge]: https://img.shields.io/travis/com/iamsauravsharma/cargo-trim.svg?logo=travis
[build_link]: https://travis-ci.com/iamsauravsharma/cargo-trim

[deps_badge]: https://deps.rs/repo/github/iamsauravsharma/cargo-trim/status.svg
[deps_link]: https://deps.rs/repo/github/iamsauravsharma/cargo-trim

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg
[license_link]: LICENSE

[loc_badge]: https://tokei.rs/b1/github/iamsauravsharma/cargo-trim
[loc_link]: https://github.com/iamsauravsharma/cargo-trim

[cratesio_link]: https://crates.io

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