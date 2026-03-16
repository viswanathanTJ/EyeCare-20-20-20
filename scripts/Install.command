#!/bin/bash
# Eye2020 Installer
# Double-click this file to install Eye2020

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="Eye2020.app"
APP_PATH="$SCRIPT_DIR/$APP_NAME"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: $APP_NAME not found in $SCRIPT_DIR"
    read -p "Press Enter to close..."
    exit 1
fi

echo "Installing Eye2020..."
echo ""

# Copy to Applications (try /Applications first, fallback to ~/Applications)
if cp -R "$APP_PATH" /Applications/ 2>/dev/null; then
    INSTALL_DIR="/Applications"
else
    mkdir -p ~/Applications
    cp -R "$APP_PATH" ~/Applications/
    INSTALL_DIR="$HOME/Applications"
fi

# Remove quarantine attribute after copying (DMG is read-only)
xattr -cr "$INSTALL_DIR/$APP_NAME"
echo "✓ Cleared macOS quarantine"
echo "✓ Installed to $INSTALL_DIR"

echo ""
echo "✓ Done! You can now open Eye2020 from Launchpad or Spotlight."
echo ""

# Open the app
open "$INSTALL_DIR/$APP_NAME"

read -p "Press Enter to close..."
