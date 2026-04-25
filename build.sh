#!/bin/bash

set -euo pipefail
# set -x

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os-$arch" in
        Darwin-arm64)  echo "macos-arm" ;;
        Darwin-x86_64) echo "macos-intel" ;;
        Linux-x86_64)  echo "linux" ;;
        *) echo "Error: unsupported platform $os-$arch" >&2; exit 1 ;;
    esac
}

PLATFORM=""
CONFIG="release"
PACKAGE=0
RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug)   CONFIG="debug" ;;
        --package) PACKAGE=1 ;;
        --run)     RUN=1 ;;
        --*)       echo "Error: unknown flag '$1'" >&2; exit 1 ;;
        *)         PLATFORM="$1" ;;
    esac
    shift
done

PLATFORM="${PLATFORM:-$(detect_platform)}"

case "$PLATFORM" in
    macos-arm)   TARGET_ARCH="aarch64-apple-darwin" ;;
    macos-intel) TARGET_ARCH="x86_64-apple-darwin" ;;
    linux)       TARGET_ARCH="x86_64-unknown-linux-gnu" ;;
    *) echo "Error: unknown platform '$PLATFORM'. Use: macos-arm | macos-intel | linux" >&2; exit 1 ;;
esac

RELEASE_OPT=()
[ "$CONFIG" == 'release' ] && RELEASE_OPT=("--release") || true

echo "Building platform=$PLATFORM arch=$TARGET_ARCH config=$CONFIG"

build_macos() {
    echo "Installing dependencies..."
    cargo binstall cargo-bundle@0.9.0 --no-confirm

    echo "Building..."
    cargo build --locked "${RELEASE_OPT[@]}" --target "$TARGET_ARCH" --features macos-app-bundle

    echo "Packing the app..."
    cargo bundle "${RELEASE_OPT[@]}" --target "$TARGET_ARCH" --package walksnail-osd-tool --features macos-app-bundle

    BUILT_APP_PATH="$PROJECT_DIR/target/$TARGET_ARCH/$CONFIG/bundle/osx/Walksnail OSD Tool.app"
    BUILT_APP_REL="target/$TARGET_ARCH/$CONFIG/bundle/osx/Walksnail OSD Tool.app"

    cp "$PROJECT_DIR/ext/ffmpeg/$PLATFORM/ffmpeg" "$BUILT_APP_PATH/Contents/MacOS/ffmpeg"
    cp "$PROJECT_DIR/ext/ffmpeg/$PLATFORM/ffprobe" "$BUILT_APP_PATH/Contents/MacOS/ffprobe"

    cd "$BUILT_APP_PATH/.."
    codesign --force -s - "Walksnail OSD Tool.app/Contents/MacOS/ffmpeg"
    codesign --force -s - "Walksnail OSD Tool.app/Contents/MacOS/ffprobe"
    codesign --force -s - "Walksnail OSD Tool.app/Contents/MacOS/walksnail-osd-tool"
    codesign --force -s - "Walksnail OSD Tool.app"

    if [ $PACKAGE -eq 1 ]; then
        DEPLOY_DIR="$PROJECT_DIR/_deploy"
        mkdir -p "$DEPLOY_DIR"
        7z a "$DEPLOY_DIR/walksnail-osd-tool-$PLATFORM.zip" "Walksnail OSD Tool.app"
        echo "Done! Artifact: $DEPLOY_DIR/walksnail-osd-tool-$PLATFORM.zip"
    elif [ $RUN -eq 1 ]; then
        open "$BUILT_APP_PATH"
    else
        echo "Done! To run: open \"$BUILT_APP_REL\""
    fi
}

build_linux() {
    echo "Building..."
    cargo build --locked "${RELEASE_OPT[@]}" --target "$TARGET_ARCH"

    if [ $PACKAGE -eq 1 ]; then
        DEPLOY_DIR="$PROJECT_DIR/_deploy"
        mkdir -p "$DEPLOY_DIR"
        cd "$PROJECT_DIR/target/$TARGET_ARCH/$CONFIG/"
        tar -czf "$DEPLOY_DIR/walksnail-osd-tool-linux.tar.gz" walksnail-osd-tool
        echo "Done! Artifact: $DEPLOY_DIR/walksnail-osd-tool-linux.tar.gz"
    else
        echo "Done! Binary: target/$TARGET_ARCH/$CONFIG/walksnail-osd-tool"
    fi
}

case "$PLATFORM" in
    macos-*) build_macos ;;
    linux)   build_linux ;;
esac
