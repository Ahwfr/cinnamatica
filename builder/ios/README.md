# Cinnamatica for iOS (experimental)

The iOS port shares the Windows/Linux placement, hologram and schematic code. The
iOS guest adds a movable **Build** touch button and a touch **Close** menu button.
The Files/iCloud picker converts supported schematics inside the app; no Windows
importer or keyboard is required. Documents contains the private library/settings.

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
