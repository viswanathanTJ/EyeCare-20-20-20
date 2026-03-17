#!/bin/bash
# Build, sign, and repack DMG with Install.command
set -e

echo "==> Ejecting any mounted Eye2020 volume..."
hdiutil detach /Volumes/Eye2020 2>/dev/null || true

echo "==> Building Tauri app..."
npx @tauri-apps/cli build

APP_PATH="src-tauri/target/release/bundle/macos/Eye2020.app"
DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" -maxdepth 1)
VOLUME_NAME="Eye2020"

echo "==> Signing app..."
codesign --force --deep --sign - "$APP_PATH"
codesign --verify --verbose "$APP_PATH"

echo "==> Unmounting any existing volume..."
hdiutil detach "/Volumes/$VOLUME_NAME" 2>/dev/null || true

echo "==> Mounting original DMG..."
MOUNT_DIR=$(hdiutil attach "$DMG_PATH" -nobrowse | grep "$VOLUME_NAME" | awk '{print $3}')
echo "    Mounted at $MOUNT_DIR"

echo "==> Unmounting original..."
hdiutil detach "$MOUNT_DIR"

echo "==> Building clean DMG contents..."
STAGING_DIR=$(mktemp -d)
cp -R "$APP_PATH" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"
cp scripts/Install.command "$STAGING_DIR/"
chmod +x "$STAGING_DIR/Install.command"

echo "==> Creating new DMG..."
rm -f "$DMG_PATH"
hdiutil create -volname "$VOLUME_NAME" -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG_PATH"
rm -rf "$STAGING_DIR"

echo ""
echo "✓ DMG ready: $DMG_PATH"
echo "==> Opening DMG..."
open "$DMG_PATH"
