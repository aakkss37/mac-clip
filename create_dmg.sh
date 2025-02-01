#!/bin/bash

# Build the release binary
cargo build --release

# Create the app bundle structure
APP_NAME="Mac Clip.app"
CONTENTS_DIR="$APP_NAME/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Create necessary directories
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy the binary
cp "target/release/mac-clip" "$MACOS_DIR/"

# Copy Info.plist
cp "Info.plist" "$CONTENTS_DIR/"

# Create DMG
DMG_NAME="Mac-Clip-0.1.0.dmg"
if [ -f "$DMG_NAME" ]; then
    rm "$DMG_NAME"
fi

# Create temporary DMG
hdiutil create -size 50m -fs HFS+ -volname "Mac Clip" -srcfolder "$APP_NAME" -format UDRW "temp.dmg"

# Convert to compressed DMG
hdiutil convert "temp.dmg" -format UDZO -o "$DMG_NAME"

# Clean up
rm "temp.dmg"
rm -rf "$APP_NAME"

echo "DMG created at $DMG_NAME"
