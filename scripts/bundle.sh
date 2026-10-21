#!/usr/bin/env bash
# Build Apature.app from the release binary. No dependencies beyond Xcode's
# command-line tools (codesign). Output: target/bundle/Apature.app
#
#   scripts/bundle.sh            # build + bundle + ad-hoc sign
#   scripts/bundle.sh --install  # also copy to /Applications (replaces an old copy)
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
BUILD=$(git rev-list --count HEAD 2>/dev/null || echo 0)
APP=target/bundle/Apature.app

APATURE_BUILD="$BUILD" cargo build --release -p apature

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/apature "$APP/Contents/MacOS/Apature"
cp apps/desktop/assets/Apature.icns "$APP/Contents/Resources/Apature.icns"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>               <string>Apature</string>
  <key>CFBundleDisplayName</key>        <string>Apature</string>
  <key>CFBundleIdentifier</key>         <string>dev.mccal.apature</string>
  <key>CFBundleVersion</key>            <string>${BUILD}</string>
  <key>CFBundleShortVersionString</key> <string>${VERSION}</string>
  <key>CFBundleExecutable</key>         <string>Apature</string>
  <key>CFBundleIconFile</key>           <string>Apature</string>
  <key>CFBundlePackageType</key>        <string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key> <string>6.0</string>
  <key>LSMinimumSystemVersion</key>     <string>12.0</string>
  <key>LSApplicationCategoryType</key>  <string>public.app-category.photography</string>
  <key>NSHighResolutionCapable</key>    <true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key> <true/>
  <key>NSHumanReadableCopyright</key>   <string>© 2026 Caleb McCartney. MIT OR Apache-2.0.</string>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key> <string>Folder</string>
      <key>CFBundleTypeRole</key> <string>Viewer</string>
      <key>LSItemContentTypes</key> <array><string>public.folder</string></array>
      <key>LSHandlerRank</key> <string>Alternate</string>
    </dict>
  </array>
</dict>
</plist>
PLIST
printf 'APPL????' > "$APP/Contents/PkgInfo"

# Ad-hoc signature so Gatekeeper lets a local build launch and the identifier
# is attached to the process (menu title, cache paths, permissions prompts).
codesign --force --deep --sign - "$APP" >/dev/null
echo "built $APP (version $VERSION, build $BUILD)"

if [[ "${1:-}" == "--install" ]]; then
  rm -rf /Applications/Apature.app
  cp -R "$APP" /Applications/Apature.app
  echo "installed /Applications/Apature.app"
fi
