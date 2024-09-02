#!/bin/bash

set -euo pipefail
# set -x

echo "🌱 Installing dependecies..."

# cargo binstall cargo-bundle

echo "🌱 Building..."

cargo build --release

echo "🌱 Done!"
