#!/bin/sh
set -e

TARGET_NAME="$1"
if [ -z "$TARGET_NAME" ]; then
  echo "$0: target name required" >&2
  exit 1
fi

mkdir -p "./fuzz/corpus/${TARGET_NAME}/"
cargo +nightly fuzz run "${TARGET_NAME}" \
  "./fuzz/corpus/${TARGET_NAME}/" "./fuzz/init_corpus/${TARGET_NAME}/" -- \
  -dict="./fuzz/dicts/${TARGET_NAME}" \
  -timeout=3
