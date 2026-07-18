#!/bin/sh

set -e

git push
cargo publish --workspace
