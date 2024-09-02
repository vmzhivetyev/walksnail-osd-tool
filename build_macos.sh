#!/bin/bash

set -euo pipefail
# set -x

# Based on: https://github.com/avsaase/walksnail-osd-tool/blob/master/.github/workflows/build.yaml

PROJECT_DIR="$(pwd)"
TARGET_NAME="macos-arm"
BUILD_TARGET_ARCH="aarch64-apple-darwin"
CONFIG_NAME="release" # debug or release

if [ $CONFIG_NAME == 'debug' ]; then
	RELEASE_OPT=""
elif [ $CONFIG_NAME == 'release' ]; then
	RELEASE_OPT="--release"
else
	echo "Error: CONFIG_NAME should be either debug or release, not \"$CONFIG_NAME\"" >&2
	exit 1
fi

echo "🌱 Installing dependecies..."

cargo binstall cargo-bundle

echo "🌱 Building..."

cargo build $RELEASE_OPT --target ${BUILD_TARGET_ARCH} --features macos-app-bundle

echo "🌱 Packing the app..."

cargo bundle $RELEASE_OPT --target ${BUILD_TARGET_ARCH} --features macos-app-bundle

BUILT_APP_PATH="./target/$BUILD_TARGET_ARCH/$CONFIG_NAME/bundle/osx/Walksnail OSD Tool.app"

cp ${PROJECT_DIR}/ext/ffmpeg/${TARGET_NAME}/ffmpeg "$BUILT_APP_PATH/Contents/MacOS/ffmpeg"
cp ${PROJECT_DIR}/ext/ffmpeg/${TARGET_NAME}/ffprobe "$BUILT_APP_PATH/Contents/MacOS/ffprobe"

cd "$BUILT_APP_PATH/.."
codesign --force -s - Walksnail\ OSD\ Tool.app/Contents/MacOS/ffmpeg
codesign --force -s - Walksnail\ OSD\ Tool.app/Contents/MacOS/ffprobe
codesign --force -s - Walksnail\ OSD\ Tool.app/Contents/MacOS/walksnail-osd-tool
codesign --force -s - Walksnail\ OSD\ Tool.app

FINAL_DIR="$PROJECT_DIR/artifacts"
mkdir -p "$FINAL_DIR"
rm -rf "$FINAL_DIR/Walksnail OSD Tool.app"
mv Walksnail\ OSD\ Tool.app "$FINAL_DIR/."

echo "🌱 Done! See $FINAL_DIR"
