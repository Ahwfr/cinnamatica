# Cinnamatica for iOS (experimental)

The iOS port stays on shared version **0.3.69**, matching Windows/Linux. Platform build commit IDs distinguish port revisions; a port alone does not bump the release version.

The iOS port shares the Windows/Linux placement, hologram and schematic code. The
iOS guest adds a movable **Build** touch button and a touch **Close** menu button.
The Files/iCloud picker converts supported schematics inside the app; no Windows
importer or keyboard is required. Documents contains the private library/settings.

The in-game Library tab loads and removes existing entries; import new files from
Settings > Mods. The iOS-only Touch tab enables individual HUD buttons for layers,
view-relative movement, rotation, mirror, visibility, placing the guide here and
position locking. Layer buttons start enabled. Choices save immediately; use the
normal touch layout editor to move or resize the enabled buttons.

Close responds in the native input handler without waiting for the interpreter.
While a repeat-enabled ghost is targeted, placement callbacks take priority over
background tile maintenance; retained geometry stays visible between updates. This
does not increase the server tick rate or bypass inventory confirmations. The
responsiveness change still needs comparison on a real device.

The iOS app uses Wasmtime's Pulley interpreter, not a JIT entitlement. It renders
through Metal and links the Go core into the app. Current Cinnabar iOS does not
provide a local-world server; joining a server/friend's world is distinct from that.

See [the port plan and device checklist](PORT-PLAN.md) for the differences from
Windows. Target test device: iPhone 15 Pro on iOS 27.0.1. Device behavior has not yet
been verified. Host checks are not a claim of iPhone acceptance.

Build on macOS/Xcode using `bash packaging/ios/build-ipa.sh`, or use the iOS GitHub
Actions workflow on an `ios/*` branch. It builds the matching iOS-touch guest and
bundles it in an unsigned, user-signable IPA. Keep the signed app identity stable
when updating so its Documents library remains available.

Public artifacts omit game assets. Follow [the asset-injection instructions](../../docs/ios.md)
using your own prepared assets before signing/sideloading. Never upload a private
profile, account tokens, worlds or Mojang-derived carriers to the public repository.
