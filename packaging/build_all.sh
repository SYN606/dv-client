#!/usr/bin/env bash
set -euo pipefail

# Digital Vigital (DV) — Master Linux Packaging Script
# Builds:
#   1. AppImage (.AppImage)
#   2. Debian package (.deb)
#   3. Red Hat / Fedora package (.rpm)

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== [1/4] Compiling optimized release binary ==="
cargo build --release

DIST_DIR="$ROOT_DIR/dist"
mkdir -p "$DIST_DIR"

STRIPPED_BIN="$DIST_DIR/dv-launcher"
strip -s -o "$STRIPPED_BIN" "$ROOT_DIR/target/release/dv-launcher"
chmod 755 "$STRIPPED_BIN"

echo "=== [2/4] Building Debian package (.deb) ==="
DEB_STAGING="$DIST_DIR/deb-staging"
rm -rf "$DEB_STAGING"
mkdir -p "$DEB_STAGING/DEBIAN" \
         "$DEB_STAGING/usr/bin" \
         "$DEB_STAGING/usr/share/applications" \
         "$DEB_STAGING/usr/share/icons/hicolor/512x512/apps" \
         "$DEB_STAGING/usr/share/pixmaps"

cp "$ROOT_DIR/packaging/deb/control" "$DEB_STAGING/DEBIAN/control"
cp "$STRIPPED_BIN" "$DEB_STAGING/usr/bin/dv-launcher"
chmod 755 "$DEB_STAGING/usr/bin/dv-launcher"
cp "$ROOT_DIR/packaging/dv-launcher.desktop" "$DEB_STAGING/usr/share/applications/dv-launcher.desktop"
chmod 644 "$DEB_STAGING/usr/share/applications/dv-launcher.desktop"
cp "$ROOT_DIR/assets/logo.png" "$DEB_STAGING/usr/share/icons/hicolor/512x512/apps/dv-launcher.png"
cp "$ROOT_DIR/assets/logo.png" "$DEB_STAGING/usr/share/pixmaps/dv-launcher.png"

cd "$DEB_STAGING"
echo "2.0" > debian-binary
tar --numeric-owner --owner=0 --group=0 -czf control.tar.gz -C DEBIAN control
tar --numeric-owner --owner=0 --group=0 -czf data.tar.gz usr
ar rcs "$DIST_DIR/dv-launcher_1.0.0_amd64.deb" debian-binary control.tar.gz data.tar.gz
cd "$ROOT_DIR"
echo "-> Created $DIST_DIR/dv-launcher_1.0.0_amd64.deb"

echo "=== [3/4] Building Red Hat / Fedora package (.rpm) ==="
if command -v rpmbuild >/dev/null 2>&1; then
    RPM_ROOT="$DIST_DIR/rpmbuild"
    rm -rf "$RPM_ROOT"
    mkdir -p "$RPM_ROOT"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    cp "$STRIPPED_BIN" "$RPM_ROOT/SOURCES/dv-launcher"
    cp "$ROOT_DIR/packaging/dv-launcher.desktop" "$RPM_ROOT/SOURCES/dv-launcher.desktop"
    cp "$ROOT_DIR/assets/logo.png" "$RPM_ROOT/SOURCES/logo.png"
    cp "$ROOT_DIR/packaging/rpm/dv-launcher.spec" "$RPM_ROOT/SPECS/dv-launcher.spec"
    rpmbuild --define "_topdir $RPM_ROOT" -bb "$RPM_ROOT/SPECS/dv-launcher.spec"
    cp "$RPM_ROOT"/RPMS/x86_64/*.rpm "$DIST_DIR/"
    cp "$DIST_DIR"/dv-launcher-1.0.0-1.*.rpm "$DIST_DIR/dv-launcher-1.0.0-1.x86_64.rpm" 2>/dev/null || true
    echo "-> Created $DIST_DIR/dv-launcher-1.0.0-1.x86_64.rpm"
else
    echo "Warning: rpmbuild not found, skipping RPM generation."
fi

echo "=== [4/4] Building portable Linux AppImage (.AppImage) ==="
APP_DIR="$DIST_DIR/AppDir"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin" \
         "$APP_DIR/usr/share/applications" \
         "$APP_DIR/usr/share/icons/hicolor/512x512/apps" \
         "$APP_DIR/usr/share/pixmaps"

cp "$ROOT_DIR/packaging/appimage/AppRun" "$APP_DIR/AppRun"
chmod 755 "$APP_DIR/AppRun"
cp "$ROOT_DIR/packaging/dv-launcher.desktop" "$APP_DIR/dv-launcher.desktop"
cp "$ROOT_DIR/packaging/dv-launcher.desktop" "$APP_DIR/usr/share/applications/dv-launcher.desktop"
cp "$ROOT_DIR/assets/logo.png" "$APP_DIR/dv-launcher.png"
cp "$ROOT_DIR/assets/logo.png" "$APP_DIR/.DirIcon"
cp "$ROOT_DIR/assets/logo.png" "$APP_DIR/usr/share/icons/hicolor/512x512/apps/dv-launcher.png"
cp "$ROOT_DIR/assets/logo.png" "$APP_DIR/usr/share/pixmaps/dv-launcher.png"
cp "$STRIPPED_BIN" "$APP_DIR/usr/bin/dv-launcher"
chmod 755 "$APP_DIR/usr/bin/dv-launcher"

if [ ! -f "$ROOT_DIR/packaging/appimagetool" ]; then
    echo "Downloading appimagetool..."
    curl -sL -o "$ROOT_DIR/packaging/appimagetool" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$ROOT_DIR/packaging/appimagetool"
fi

ARCH=x86_64 "$ROOT_DIR/packaging/appimagetool" --appimage-extract-and-run "$APP_DIR" "$DIST_DIR/DigitalVigital-1.0.0-x86_64.AppImage"
echo "-> Created $DIST_DIR/DigitalVigital-1.0.0-x86_64.AppImage"

echo "=== All Linux distribution packages generated successfully in dist/ ==="
ls -lh "$DIST_DIR"/*.AppImage "$DIST_DIR"/*.deb "$DIST_DIR"/*.rpm
