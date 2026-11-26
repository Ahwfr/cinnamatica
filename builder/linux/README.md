# Cinnamatica for Linux

The Linux port, its launch scripts, the schematic picker, the AppImage and the `.deb` are by [ThsSilentKnight](https://github.com/ThsSilentKnight).

## Install

Download one of these from [Cinnamatica Releases](https://github.com/Ahwfr/cinnamatica/releases).

### AppImage (any distribution)

```bash
chmod +x Cinnamatica-*-x86_64.AppImage
./Cinnamatica-*-x86_64.AppImage
```

If FUSE isn't available, add `--appimage-extract-and-run`. To update, replace the file.

### `.deb` (Debian, Ubuntu, Mint, Pop!_OS)

```bash
sudo apt install ./cinnamatica_*_amd64.deb
```

Then start **Cinnamatica** from the app menu, or run `cinnamatica`. It installs to `/opt/cinnamatica`. Remove it with `sudo apt remove cinnamatica`.

### What both need

- A 64-bit distribution with a glibc at least as new as the build machine's. The `.deb` checks this when you install it.
- A Vulkan driver.
- **Zenity** or **KDialog** for the schematic picker.

Both packages keep your worlds, login, options and schematic library in `~/.local/share/Cinnamatica` (or `$XDG_DATA_HOME/Cinnamatica`), so you can switch between them freely. Set `CINNABAR_USER_ROOT` to use another folder. The first launch prepares Minecraft assets from your own game files.

## Build from source

Install the pinned Rust and Go toolchains, GNU Make, and the development packages in the [Linux port notes](../../docs/PORTING.md). Then:

```bash
git clone https://github.com/Ahwfr/cinnamatica.git
cd cinnamatica
builder/linux/build-builder.sh
builder/linux/start-builder.sh
```

## Build the packages

Run these from a clean checkout. The packages record the commit they were built from.

1. Build the release binaries and the schematic importer:
   ```bash
   make package-binaries
   builder/linux/build-builder.sh --importer-only
   ```
2. Build the AppImage. This needs `appimagetool` on the `PATH`, or `APPIMAGETOOL` set to it:
   ```bash
   builder/linux/build-appimage.sh
   ```
3. Build the `.deb`. This needs `dpkg-deb` and `fakeroot`:
   ```bash
   builder/linux/build-deb.sh
   ```

Both land in `.local/dist/cinnamatica-linux/`. The release workflow pins `appimagetool` 1.9.1 and the AppImage runtime with checksums in `.github/workflows/package.yml`.
