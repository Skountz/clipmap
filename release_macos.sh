#!/bin/sh
# clipmap — build, sign, package, notarize
# One-time setup: see README or top of this file.
set -e

# ── CONFIG (edit these) ───────────────────────────────────────────────────────
SIGN_ID="Developer ID Application: Your Name (XXXXXXXXXX)"  # from: security find-identity -v -p codesigning
NOTARY_PROFILE="clipmap-notary"   # name used in: notarytool store-credentials
# ─────────────────────────────────────────────────────────────────────────────

APP="clipmap.app"
VERSION=$(cargo metadata --no-deps --format-version 1 \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])")
DMG="clipmap-${VERSION}.dmg"

echo "clipmap v${VERSION}"

# ── BUILD ─────────────────────────────────────────────────────────────────────
echo "▸ Building…"
cargo build --release

# ── BUNDLE ────────────────────────────────────────────────────────────────────
echo "▸ Creating app bundle…"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp Info.plist          "$APP/Contents/"
cp target/release/clipmap "$APP/Contents/MacOS/"
cp config.json         "$APP/Contents/MacOS/"

# ── SIGN ──────────────────────────────────────────────────────────────────────
echo "▸ Signing…"
codesign --deep --force --options runtime \
  --timestamp \
  --sign "$SIGN_ID" \
  "$APP"
codesign --verify --deep --strict "$APP"

# ── DMG ───────────────────────────────────────────────────────────────────────
echo "▸ Creating DMG…"
rm -f "$DMG"
hdiutil create -volname "clipmap" -srcfolder "$APP" -ov -format UDZO "$DMG"

# ── NOTARIZE ──────────────────────────────────────────────────────────────────
echo "▸ Notarizing (takes ~1 min)…"
xcrun notarytool submit "$DMG" \
  --keychain-profile "$NOTARY_PROFILE" \
  --wait

# ── STAPLE ────────────────────────────────────────────────────────────────────
echo "▸ Stapling…"
xcrun stapler staple "$DMG"

echo "✓  $DMG is ready to distribute"
