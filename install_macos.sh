#!/bin/sh
# clipmap — macOS install
# Creates an app bundle so notifications work, then registers a LaunchAgent.
set -e

APP=/Applications/clipmap.app
PLIST=~/Library/LaunchAgents/fr.frenchbytes.clipmap.plist

echo "Building…"
cargo build --release

echo "Creating bundle at $APP…"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"
cp Info.plist      "$APP/Contents/"
cp target/release/clipmap "$APP/Contents/MacOS/"
cp config.json "$APP/Contents/Resources/"
# Copy any local mappings file referenced in config
MAPPINGS=$(python3 -c "import json,sys; print(json.load(open('config.json'))['mappings_url'])" 2>/dev/null || true)
if [ -n "$MAPPINGS" ] && [ -f "$MAPPINGS" ]; then
    cp "$MAPPINGS" "$APP/Contents/Resources/"
fi

echo "Installing LaunchAgent…"
cp macos.plist "$PLIST"
launchctl unload "$PLIST" 2>/dev/null || true
launchctl load   "$PLIST"

echo "Done. clipmap is running."
echo "To check: launchctl list | grep clipmap"
