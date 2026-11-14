#!/usr/bin/env bash
# android-setup.sh — Install everything `dx bundle --platform android` needs:
#   Rust Android targets, Android SDK cmdline-tools, platform, build-tools, NDK.
# Then build the APK with:   ./scripts/android-setup.sh build
#
# Downloads ~3 GB into $ANDROID_HOME (default ~/Android/Sdk). Safe to re-run.

set -euo pipefail

ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
NDK_VERSION="27.2.12479018"
PLATFORM="android-34"
BUILD_TOOLS="34.0.0"
CMDLINE_TOOLS_ZIP="commandlinetools-linux-11076708_latest.zip"
PORTABLE_JDK="$(dirname "$ANDROID_HOME")/jdk-17"

export ANDROID_HOME
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"
export NDK_HOME="$ANDROID_NDK_HOME"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

find_jdk17() {
    for d in "${JAVA_HOME:-}" "$PORTABLE_JDK" /usr/lib/jvm/java-17-openjdk /usr/lib/jvm/java-17-openjdk-amd64 /usr/lib/jvm/temurin-17*; do
        [ -n "$d" ] && [ -x "$d/bin/java" ] && "$d/bin/java" -version 2>&1 | grep -q '"17\.' && { echo "$d"; return; }
    done
}

setup() {
    echo "== Rust Android targets =="
    rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

    echo "== JDK 17 (Gradle used by dx 0.6 doesn't support newer JDKs) =="
    if [ -z "$(find_jdk17)" ]; then
        # No system JDK 17 (e.g. pacman's jdk17-openjdk) — fetch a portable
        # Eclipse Temurin 17 build; no root needed.
        echo "No JDK 17 found — downloading Temurin 17 to $PORTABLE_JDK"
        arch="$(uname -m)"; [ "$arch" = "x86_64" ] && arch="x64"
        tmp="$(mktemp -d)"
        curl -fL "https://api.adoptium.net/v3/binary/latest/17/ga/linux/$arch/jdk/hotspot/normal/eclipse" -o "$tmp/jdk.tar.gz"
        mkdir -p "$PORTABLE_JDK"
        tar -xzf "$tmp/jdk.tar.gz" -C "$PORTABLE_JDK" --strip-components=1
        rm -rf "$tmp"
    fi

    echo "== Android cmdline-tools → $ANDROID_HOME =="
    if [ ! -x "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]; then
        tmp="$(mktemp -d)"
        curl -fL "https://dl.google.com/android/repository/$CMDLINE_TOOLS_ZIP" -o "$tmp/tools.zip"
        unzip -q "$tmp/tools.zip" -d "$tmp"
        mkdir -p "$ANDROID_HOME/cmdline-tools"
        rm -rf "$ANDROID_HOME/cmdline-tools/latest"
        mv "$tmp/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
        rm -rf "$tmp"
    fi

    echo "== SDK packages (you'll be asked to accept Google's licenses) =="
    export JAVA_HOME="$(find_jdk17)"
    sdkmanager --licenses
    sdkmanager "platform-tools" "platforms;$PLATFORM" "build-tools;$BUILD_TOOLS" "ndk;$NDK_VERSION"

    cat <<EOF

Done. Add these to your shell profile (~/.zshrc) so dx finds them:

  export JAVA_HOME="$JAVA_HOME"
  export ANDROID_HOME="$ANDROID_HOME"
  export ANDROID_NDK_HOME="$ANDROID_NDK_HOME"
  export NDK_HOME="\$ANDROID_NDK_HOME"
  export PATH="\$ANDROID_HOME/platform-tools:\$PATH"

Then build:  ./scripts/android-setup.sh build
EOF
}

build() {
    export JAVA_HOME="${JAVA_HOME:-$(find_jdk17)}"
    [ -d "$ANDROID_NDK_HOME" ] || { echo "NDK missing — run ./scripts/android-setup.sh first"; exit 1; }

    echo "== Building APK (arm64) =="
    dx bundle --platform android --release \
        --target aarch64-linux-android \
        --no-default-features --features mobile

    # dx leaves the debug-signed (installable) APK deep in target/; copy it out.
    version="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
    apk="$(find target/dx -path '*outputs/apk/*' -name '*.apk' -newer Cargo.toml | head -1)"
    [ -n "$apk" ] || { echo "No APK found under target/dx"; exit 1; }
    mkdir -p dist
    cp "$apk" "dist/git-tree-$version-arm64.apk"
    echo
    echo "APK: dist/git-tree-$version-arm64.apk"
    echo "Install on a phone with USB debugging on:"
    echo "  adb install -r dist/git-tree-$version-arm64.apk"
}

case "${1:-setup}" in
    setup) setup ;;
    build) build ;;
    *) echo "usage: $0 [setup|build]"; exit 1 ;;
esac
