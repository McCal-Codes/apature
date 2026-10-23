#!/usr/bin/env bash
# Build the Linux release: an AppImage (one file, runs on most distributions)
# and a tar.gz with the same contents, both named by version and architecture.
#
#   scripts/package-linux.sh            # target/bundle/Apature-<v>-<arch>.AppImage + .tar.gz
#
# Needs: cargo, libgtk-3-dev, libdbus-1-dev, and `appimagetool` on PATH (or
# APPIMAGETOOL pointing at it). Without appimagetool only the tar.gz is made.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
VERSION=$(grep -m1 '^version = ' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
ARCH=$(uname -m)
OUT=target/bundle
APPDIR=$OUT/Apature.AppDir
export APATURE_BUILD=${APATURE_BUILD:-$(git rev-list --count HEAD)}

cargo build --release -p apature -p apature-cli

rm -rf "$APPDIR" && mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$APPDIR/usr/share/icons/hicolor/512x512/apps"
cp target/release/apature "$APPDIR/usr/bin/apature"
cp target/release/apature-cli "$APPDIR/usr/bin/apature-cli" 2>/dev/null || true
cp apps/desktop/assets/linux/dev.mccal.Apature.desktop "$APPDIR/usr/share/applications/"
cp apps/desktop/assets/linux/dev.mccal.Apature.desktop "$APPDIR/dev.mccal.Apature.desktop"
cp docs/design/icons/modern-a-dark/modern-a-dark-256.png "$APPDIR/usr/share/icons/hicolor/256x256/apps/dev.mccal.Apature.png"
cp docs/design/icons/modern-a-dark/modern-a-dark-512.png "$APPDIR/usr/share/icons/hicolor/512x512/apps/dev.mccal.Apature.png"
cp docs/design/icons/modern-a-dark/modern-a-dark-256.png "$APPDIR/dev.mccal.Apature.png"
cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/apature" "$@"
RUN
chmod +x "$APPDIR/AppRun"

# tar.gz: the same tree, for people who prefer not to run AppImages
TAR=$OUT/Apature-$VERSION-linux-$ARCH.tar.gz
rm -f "$TAR"
tar -C "$APPDIR" -czf "$TAR" usr AppRun dev.mccal.Apature.desktop dev.mccal.Apature.png
echo "wrote $TAR"

# .deb for Debian and Ubuntu: the same files under /usr, a desktop entry,
# icons, and the GTK and D-Bus libraries as dependencies.
if command -v dpkg-deb >/dev/null 2>&1; then
  DEBARCH=$(dpkg --print-architecture 2>/dev/null || echo amd64)
  DEBDIR=$OUT/deb
  rm -rf "$DEBDIR" && mkdir -p "$DEBDIR/DEBIAN" "$DEBDIR/usr"
  cp -R "$APPDIR/usr/." "$DEBDIR/usr/"
  cat > "$DEBDIR/DEBIAN/control" <<CTRL
Package: apature
Version: $VERSION
Section: graphics
Priority: optional
Architecture: $DEBARCH
Depends: libgtk-3-0, libdbus-1-3, libxkbcommon0, libegl1
Maintainer: Caleb McCartney <contact@mcc-cal.com>
Homepage: https://github.com/McCal-Codes/apature
Description: Contact sheet, culling, captions, export and delivery for photojournalists
 Apature rebuilds the Photo Mechanic loop (card in, cull, caption, export,
 send) as a native app. Originals are never modified.
CTRL
  DEB=$OUT/apature_${VERSION}_${DEBARCH}.deb
  dpkg-deb --build --root-owner-group "$DEBDIR" "$DEB" >/dev/null
  echo "wrote $DEB"
fi

TOOL=${APPIMAGETOOL:-$(command -v appimagetool || true)}
if [ -n "$TOOL" ]; then
  APPIMAGE=$OUT/Apature-$VERSION-linux-$ARCH.AppImage
  rm -f "$APPIMAGE"
  ARCH=$ARCH "$TOOL" --no-appstream "$APPDIR" "$APPIMAGE" >/dev/null
  echo "wrote $APPIMAGE"
else
  echo "appimagetool not found; AppImage skipped" >&2
fi
