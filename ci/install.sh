#!/usr/bin/env bash
#shellcheck disable=SC2015
#SC2015 => Note that A && B || C is not if-then-else. C may run when A is true.
#SC2015 => https://github.com/koalaman/shellcheck/wiki/SC2015
set -ex

# if $TARGET env is present install cross else install rustfmt and clippy
if [[ -n $TARGET ]]
then
    rustup target add "$TARGET"
else
  # add rustfmt and clippy to target
  rustup component add rustfmt && export RUSTFMT_ADDED="true" || export RUSTFMT_ADDED="false"
  rustup component add clippy && export CLIPPY_ADDED="true" || export CLIPPY_ADDED="false"
fi

set +x