#!/usr/bin/env bash
# Builds cinnamatica_<version>_<arch>.deb from the same staged payload as the AppImage
# (builder/linux/build-appimage.sh with APPDIR_ONLY=1), installed under /opt/cinnamatica with a
# `cinnamatica` command, a menu entry and an icon.
# Usage: builder/linux/build-deb.sh [out_dir]
# Env: the build-appimage.sh variables (BINARIES, ASSETC, ARCH).
set -euo pipefail
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
repo_root="$(CDPATH= cd -- "$here/../.." && pwd)"
out="${1:-$repo_root/.local/dist/cinnamatica-linux}"

appdir="$(APPDIR_ONLY=1 "$here/build-appimage.sh" "$out" | tail -n 1)"
version="$(cat "$appdir/App/builder-version.txt")"
case "${ARCH:-$(uname -m)}" in
    x86_64) arch=amd64 ;;
    aarch64 | arm64) arch=arm64 ;;
    *) echo "No Debian architecture for ${ARCH:-$(uname -m)}." >&2; exit 1 ;;
esac
# The newest glibc symbol version the client needs, so dpkg refuses systems that cannot run it.
glibc="$(objdump -T "$appdir/App/bin/cinnabar" | grep -o 'GLIBC_[0-9.]*' | sort -Vu | tail -n 1)"
glibc="${glibc#GLIBC_}"

root="$out/deb-root"
rm -rf "$root"
install -d "$root/DEBIAN" "$root/opt/cinnamatica" "$root/usr/bin" \
    "$root/usr/share/applications" "$root/usr/share/icons/hicolor/scalable/apps" \
    "$root/usr/share/doc/cinnamatica"
cp -a "$appdir/." "$root/opt/cinnamatica/"
rm -f "$root/opt/cinnamatica/AppRun" "$root/opt/cinnamatica/.DirIcon"

# Same user data folder as the AppImage, so both share worlds, login and the schematic library.
cat > "$root/usr/bin/cinnamatica" <<'EOF'
#!/bin/sh
: "${CINNABAR_USER_ROOT:=${XDG_DATA_HOME:-$HOME/.local/share}/Cinnamatica}"
export CINNABAR_USER_ROOT
exec /opt/cinnamatica/Cinnamatica "$@"
EOF
chmod 0755 "$root/usr/bin/cinnamatica"
sed 's/^Exec=.*/Exec=cinnamatica/' "$here/cinnamatica.desktop" > "$root/usr/share/applications/cinnamatica.desktop"
install -m 0644 "$repo_root/packaging/icons/cinnabar.svg" "$root/usr/share/icons/hicolor/scalable/apps/cinnamatica.svg"
install -m 0644 "$repo_root/LICENSE" "$root/usr/share/doc/cinnamatica/copyright"

size="$(du -sk --exclude=DEBIAN "$root" | cut -f1)"
cat > "$root/DEBIAN/control" <<EOF
Package: cinnamatica
Version: $version
Architecture: $arch
Maintainer: ThsSilentKnight and Ahwfr <https://github.com/Ahwfr/cinnamatica>
Installed-Size: $size
Depends: libc6 (>= $glibc), libgcc-s1, libasound2t64 | libasound2, libudev1, libvulkan1, libxkbcommon0
Recommends: libwayland-client0, libx11-6, libxcursor1, libxi6, libxrandr2, zenity | kdialog
Section: games
Priority: optional
Homepage: https://github.com/Ahwfr/cinnamatica
Description: Minecraft Bedrock client with schematic building
 Cinnamatica adds schematic guides, Fast Place and Air Place to the
 Cinnabar client for Minecraft: Bedrock Edition. It ships no Minecraft
 files and needs a Microsoft account that owns the game.
EOF

deb="$out/cinnamatica_${version}_${arch}.deb"
fakeroot dpkg-deb --build --root-owner-group "$root" "$deb" >/dev/null
rm -rf "$root"
echo "$deb"
