#!/usr/bin/env bash

mkdir -p dist

TARGET=${TARGET:-x86_64-apple-darwin}
TAG=${TAG:-canary}
OS_ARCH_EXT=${OS_ARCH_EXT:-darwin-amd64}

cargo build --release --target "${TARGET}"
mv "target/${TARGET}/release/gh-form" "./dist/gh-form_${TAG}_${OS_ARCH_EXT}"
