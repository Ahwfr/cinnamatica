# Cinnamatica

**Schematic building for Minecraft: Bedrock Edition**, built on the [Cinnabar](https://github.com/cinnabar-mc/cinnabar) client.

Import a Java schematic or a Bedrock structure, see it as a hologram in your world, and build it by hand or with Fast Place.

> [!IMPORTANT]
> **Not an official Minecraft product.** Not approved by or associated with Mojang or Microsoft. Minecraft is a trademark of Microsoft Corporation; its code and assets belong to Mojang and Microsoft under the [Minecraft EULA](https://www.minecraft.net/eula). Cinnamatica ships no Minecraft files. You need your own copy of the game, and packaged launches check that your Microsoft account owns it.

## Features

- **Schematic import:** `.schematic`, `.schem`, `.litematic` and Bedrock `.mcstructure` files go into a library you can load at any time.
- **Holograms:** missing and incorrect blocks show as ghosts, with materials lists and layer-by-layer views.
- **Fast Place:** places the planned block, with the right item and orientation, from a nearby support face.
  - **V1 Classic** clicks real supports only.
  - **V2 Strict** checks the planned block state and can place compatible blocks straight into air.
- **Hold to Place:** repeats Fast Place while you hold the use button.
- **Target info:** a readout of the planned block, the real world block and Fast Place's prediction. Debug mode explains why a placement won't work.
- **Container highlights:** chests and shulkers holding blocks you still need are outlined.
- **Paste build:** places a whole schematic at once (Creative, operator only).

Press **M** in-game to open the schematic menu. The [Cinnamatica guide](builder/README.md) covers every control and setting.

## Install

| Platform | Download | Build from source |
| --- | --- | --- |
| **Windows** | Installer or portable ZIP from [Releases](https://github.com/Ahwfr/cinnamatica/releases) | [Windows guide](builder/windows/README.md) |
| **Linux** | `.AppImage` (any distribution) or `.deb` (Debian, Ubuntu) from [Releases](https://github.com/Ahwfr/cinnamatica/releases) | [Linux guide](builder/linux/README.md) |
| **iOS** | Not available yet | [Port status](builder/ios/README.md) |
| **Android** | Not available yet | [Port status](builder/android/README.md) |

On first launch, Cinnamatica prepares the Minecraft assets it needs from your own game files.

## How it works

Cinnamatica is Cinnabar with a building layer on top:

| Part | What it does | Where |
| --- | --- | --- |
| **Cinnabar client** | The Bedrock client itself: rendering, world, input and networking (Rust, with a Go core for Xbox sign-in and transport). | [`app`](app), [`crates`](crates), [`core`](core) |
| **Schematica mod** | The schematic library, holograms, menus and Fast Place planning. It runs as a sandboxed WebAssembly mod through Cinnabar's mod API. | [`builder/schematica`](builder/schematica) |
| **Launcher and packaging** | Builds, starts and packages Cinnamatica for each platform. | [`builder`](builder) |

When you Fast Place, the mod chooses the target ghost block, and the client picks a support face, gets the item into your hand and sends a normal Bedrock placement, the same packets a vanilla client sends. The server always has the final say.

## Documentation

- [Cinnamatica guide](builder/README.md): controls, settings and Fast Place modes
- [Release notes](builder/RELEASE-NOTES.md) and [update notes](builder/UPDATE.md)
- [Contributing](CONTRIBUTING.md)

## Credits

| Contributor | Work |
| --- | --- |
| **[Ahwfr](https://github.com/Ahwfr)** | Creator and maintainer of Cinnamatica: the Schematica mod, schematic import, holograms, Fast Place and the Windows releases. |
| **[ThsSilentKnight](https://github.com/ThsSilentKnight)** | The Linux port (build and launch scripts, schematic picker, AppImage and `.deb`), the hologram overhaul, the Cinnamatica window, Target info Debug mode and many Fast Place fixes. |
| **[Cinnabar](https://github.com/cinnabar-mc/cinnabar) contributors** | The client Cinnamatica is built on. |

## License

Cinnamatica keeps Cinnabar's [PolyForm Noncommercial license](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md). The Schematica mod's own license is in [`builder/schematica/SCHEMATICA-LICENSE.txt`](builder/schematica/SCHEMATICA-LICENSE.txt).
