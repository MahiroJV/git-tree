#!/usr/bin/env bash

set -e

APP_NAME="git-tree"
BINARY_NAME="git-tree"

echo "== Building release binary =="

cargo build --release

echo "== Preparing AppDir =="

rm -rf AppDir
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/icons/hicolor/256x256/apps

cp target/release/$BINARY_NAME AppDir/usr/bin/

cp assets/icon/icon.png \
   AppDir/usr/share/icons/hicolor/256x256/apps/$APP_NAME.png

cat > AppDir/$APP_NAME.desktop <<EOF
[Desktop Entry]
Name=git-tree
Exec=$BINARY_NAME
Icon=$APP_NAME
Type=Application
Categories=Development;
EOF

echo "== Downloading appimagetool =="

if [ ! -f appimagetool.AppImage ]; then
    wget -O appimagetool.AppImage \
    https://github.com/AppImage/AppImageKit/releases/latest/download/appimagetool-x86_64.AppImage

    chmod +x appimagetool.AppImage
fi

echo "== Building AppImage =="

ARCH=x86_64 ./appimagetool.AppImage AppDir

echo "Done!"