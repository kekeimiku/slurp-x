#!/bin/bash

project="slurp-x"

targets=(
  "x86_64-unknown-linux-gnu"
)

mkdir -p build

for target in "${targets[@]}"; do
  rustup target add "$target"
  cargo build --target="$target"
  cp target/"$target"/debug/"$project" build/"$project"_"$target"
done
