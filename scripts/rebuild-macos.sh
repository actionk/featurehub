#!/usr/bin/env bash
# Rebuild Feature Hub in release mode and restart it on macOS.
# Usage: ./scripts/rebuild-macos.sh

set -euo pipefail

APP_NAME="Feature Hub"
BUNDLE_ID="com.littlebrushgames.feature-hub"
APP_BUNDLE="/Applications/${APP_NAME}.app"

echo "==> Killing running instance..."
pkill -f "${APP_BUNDLE}" 2>/dev/null && sleep 1 || true

echo "==> Building release..."
npm run tauri build 2>&1

# Find the built .app bundle
BUILD_APP="src-tauri/target/release/bundle/macos/${APP_NAME}.app"
if [ ! -d "$BUILD_APP" ]; then
  echo "ERROR: Built app not found at ${BUILD_APP}"
  exit 1
fi

echo "==> Installing to /Applications..."
rm -rf "${APP_BUNDLE}"
cp -R "${BUILD_APP}" "${APP_BUNDLE}"

echo "==> Installing CLI binaries..."
node scripts/install-cli.mjs --release

echo "==> Launching ${APP_NAME}..."
open "${APP_BUNDLE}"

echo "Done!"
