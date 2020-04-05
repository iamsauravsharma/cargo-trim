#!/usr/bin/env bash/
#shellcheck disable=SC2086
#SC2086 => Double quote to prevent globbing and word splitting
#SC2086 => https://github.com/koalaman/shellcheck/wiki/SC2086
set -ex

run_all_cargo_command(){
  if [[ $RUSTFMT_ADDED == "false" ]]
  then
    cargo +nightly-"${LAST_AVAILABLE_FMT}" fmt --all $FEATURES -- --check
  else
    cargo fmt --all $FEATURES -- --check
  fi

  if [[ $CLIPPY_ADDED == "false" ]]
  then
    cargo +nightly-"${LAST_AVAILABLE_CLIPPY}" clippy --all $FEATURES -- -D warnings
  else
    cargo clippy --all $FEATURES -- -D warnings
  fi

  cargo build --all $FEATURES
  cargo doc --no-deps --all $FEATURES
  cargo test --all $FEATURES
}

# if $TARGET is present then run only build over that target else run_all_cargo_command
if [[ -n $TARGET ]]
then
    cargo build --all --target="$TARGET" $FEATURES
else
  run_all_cargo_command
fi

set +x