# CARGO-TRIM

**Project status & info:**

|                    License                     |              Crates Version               |
| :--------------------------------------------: | :---------------------------------------: |
| [![License: MIT][license_badge]][license_link] | [![Crate][cratesio_badge]][cratesio_link] |

`cargo-trim` cleans up the **\$CARGO_HOME** cache (`registry` and `git`, default `~/.cargo`) and
the `target` folders of your Rust projects.

### Install

```
cargo install cargo-trim
```

Latest development version: `cargo install --git https://github.com/iamsauravsharma/cargo-trim`

### Quick start

```bash
cargo trim set -d ~/projects   # folder holding your projects, `.` adds current folder
cargo trim -n -x               # preview removal of crates no project uses
cargo trim -x                  # remove them
```

Add `-n` (`--dry-run`) to any command to see what would happen without changing anything.

### Terms

- **orphan**: a crate not used by any `Cargo.lock` found in project directories. Without any project
  directory every crate is orphan, so cargo-trim asks before cleaning.
- **old**: an older version of a registry crate which also has a newer version, or a git checkout of
  a revision which is not the latest fetched one.
- **stale**: a project without any change for `stale_days` days. Its `Cargo.lock` no longer marks
  crates as used and its target folder can be cleaned with `cargo trim target -s`.

### Common commands

| Command | Effect |
| --- | --- |
| `cargo trim -q` | Show size of cache and target folders |
| `cargo trim -x` | Clean orphan crates |
| `cargo trim -o` | Clean old crates |
| `cargo trim -O` | Clean crates which are both old and orphan |
| `cargo trim -a` | Clean all crates, empty indexes and target folders |
| `cargo trim -l` | Remove sources but keep archives for offline builds |
| `cargo trim list -x` | List orphan crates |
| `cargo trim list -k` | List crates kept by filter |
| `cargo trim list -p` | List detected projects |
| `cargo trim target -s` | Clean target folders of stale projects |
| `cargo trim config -p` | Print config |

`git`, `registry` and `target` subcommands limit an operation to that part. Run `cargo trim -h` for
a short overview and `cargo trim --help` (or `cargo trim <command> --help`) for details.

```
Clean up $CARGO_HOME cache and target folders of rust projects

Usage: cargo-trim [OPTIONS] [COMMAND]

Commands:
  config    Show config file location or content
  set       Add or change config values
  unset     Remove config values
  list      List crates and projects
  git       Operate only on git cache
  registry  Operate only on registry cache
  target    Operate only on target folders of projects
  help      Print this message or the help of the given subcommand(s)

Options:
  -a, --all                        Clean all crates, empty indexes and target folders
  -d, --directory <path>           Add project directory for this run [env: TRIM_DIRECTORY=]
  -n, --dry-run                    Show what would be done without changing anything
  -g, --gc <kind>                  Git compress cache repositories (needs git) [possible values: aggressive-checkout, aggressive-db, aggressive-index, checkout, db, index]
  -i, --ignore <path>              Ignore path while scanning for this run [env: TRIM_IGNORE=]
  -f, --filter <spec>              Add crate filter for this run [env: TRIM_FILTER=]
  -l, --light                      Remove sources but keep archives for offline builds
  -o, --old                        Clean old crates
  -O, --old-orphan                 Clean crates which are both old and orphan
  -x, --orphan                     Clean orphan crates
  -q, --query                      Show size of cache and target folders
  -H, --scan-hidden-folder <bool>  Scan hidden folders for this run [env: TRIM_SCAN_HIDDEN_FOLDER=] [possible values: true, false]
  -T, --scan-target-folder <bool>  Scan target folders for Cargo.lock files for this run [env: TRIM_SCAN_TARGET_FOLDER=] [possible values: true, false]
  -s, --stale-days <days>          Days without change after which a project is stale for this run [env: TRIM_STALE_DAYS=]
  -t, --top <number>               Show given number of largest crates
  -u, --update                     Run `cargo update` in every detected project
  -w, --wipe <folder>              Delete whole cache folder [possible values: git, checkouts, db, registry, cache, index, index-cache, src]
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

### Configuration

Settings live in `cargo_trim_config.toml` in the OS config directory (`cargo trim config -l`). Change
them with `cargo trim set` / `cargo trim unset`, or override them for one run with the matching
option or environment variable.

| Key | Default | Option | Env | Meaning |
| --- | --- | --- | --- | --- |
| `directory` | `[]` | `-d <path>` | `TRIM_DIRECTORY` | Folders scanned for projects, stored as absolute paths |
| `ignore` | `[]` | `-i <path>` | `TRIM_IGNORE` | Paths skipped while scanning |
| `filter` | `[]` | `-f <spec>` | `TRIM_FILTER` | Crates which may or may never be cleaned, `[]` allows every crate |
| `scan_hidden_folder` | `false` | `-H <bool>` | `TRIM_SCAN_HIDDEN_FOLDER` | Scan folders starting with `.` |
| `scan_target_folder` | `false` | `-T <bool>` | `TRIM_SCAN_TARGET_FOLDER` | Scan target folders for `Cargo.lock` |
| `stale_days` | `0` | `-s <days>` | `TRIM_STALE_DAYS` | Days without change until a project is stale, `0` is off |

Target folders are located like cargo does, through `CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR`,
`build.target-dir` in `.cargo/config.toml` or the default `target`.

#### ignore

- a **relative path** such as `node_modules` or `crates/demo` matches the end of a path anywhere.
- an **absolute path** such as `/home/user/projects/demo` matches only that path.
- each path component may contain `*`: `*_old`, `crates/*-demo`, `/home/*/scratch`.
- a folder holding a file or folder named `.cargo-trim-ignore` is always skipped, no config needed.

Ignored folders count for neither used crates nor target cleaning.

```bash
cargo trim set -i node_modules -i '*_old'
touch ~/projects/old-experiment/.cargo-trim-ignore
```

#### filter

Each entry is a package spec `[registry:]name[@version]`. The default `[]` allows every crate to be
cleaned.

- a **plain entry** such as `tokio*` restricts cleaning to matching crates. Without any plain entry
  every crate may be cleaned.
- an entry starting with **`!`** such as `!serde` is never cleaned and wins over plain entries. A
  registry still holding such a crate keeps its index on `--all`.

```toml
filter = ["!serde"]                                       # clean everything except serde
filter = ["tokio*"]                                       # clean only tokio crates
filter = ["my-company.com:*", "!my-company.com:internal"] # clean only company registry, except internal
```

Spec parts:

- **registry** is optional and matches the registry folder name with or without its hash suffix,
  so `index.crates.io:serde` matches `serde` from `index.crates.io-1949cf8c6b5b557f`. It may contain
  `*`: `!my-company.com:*` never cleans anything from that registry.
- **name** may contain `*`: `windows-*`, `*-sys`. For git crates the repository name is matched.
- **version** is a semver requirement such as `tokio@^1` or `syn@<2`, a full version such as
  `syn@1.0.109` matches exactly and for git crates it is a revision prefix such as `repo@abc1234`.

The filter applies to `--all`, `--old`, `--orphan`, `--old-orphan` and the matching `list` options.
Every clean reports how many crates the filter kept, such as `3 orphan crates removed which had
occupied 6.000 MB, 2 kept by filter`, and `cargo trim list -k` lists them.

```bash
cargo trim set -f '!serde' -f '!my-company.com:*'
cargo trim -n -x -f 'index.crates.io:tokio*' -f '!tokio@^1'
```

Entries passed with `-f` apply to that run only. Quote `!` entries so the shell does not treat `!` as
history expansion.

[license_badge]: https://img.shields.io/github/license/iamsauravsharma/cargo-trim.svg?style=for-the-badge
[license_link]: LICENSE
[cratesio_badge]: https://img.shields.io/crates/v/cargo-trim.svg?style=for-the-badge
[cratesio_link]: https://crates.io/crates/cargo-trim
