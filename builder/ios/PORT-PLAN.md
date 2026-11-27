# Cinnamatica iOS port

Target device: iPhone 15 Pro, iOS 27.0.1. Shared baseline: c4db4a919 / Builder 0.3.69.
Use the existing Cinnabar iOS application, not a separate game implementation.

## Changes and acceptance

1. **Execution:** use Wasmtime Pulley on iOS, retaining fuel, memory, capability and
   callback deadlines. No executable-memory JIT, spawned importer or developer JIT
   entitlement. Validate the pinned runtime on an actual arm64 device.
2. **Mod packaging:** build the same guest with an iOS-only touch feature; bundle
   its matching template/component and version. Install/refresh it inside Documents
   while preserving imported library payload, settings, saved placements and grants.
3. **Touch:** a movable/resizable Build HUD button opens the existing menu without
   M. Include a touch Close button. Keep desktop bindings and Android unchanged.
   Verify movement/look/use multi-touch, hold release, and pause/background cleanup.
4. **Import:** reuse the bounded Rust schematic decoder and component/library writer
   in process. Use UIKit Files picker (including iCloud providers), coordinated
   bounded copies into the app sandbox, cancellation and RAII cleanup. Never spawn
   the Windows importer or preserve external provider URLs as ordinary paths.
5. **Small screen:** safe-area-aware landscape layout, finger-size targets, native
   keyboard for search, explicit scope/page/layer/movement buttons. Test both iPhone
   landscape orientations, display scale and keyboard dismissal. No hover dependency.
6. **Rendering:** use the shared retained full-guide renderer through Metal. Keep
   complete-build behavior, exact models/outlines and incremental updates. Measure
   RAM/GPU uploads, large palettes and thermal/FPS behavior on the target phone;
   do not silently turn full rendering into a moving window to hide memory problems.
7. **Worlds/actions:** retain ordinary inventory swaps and server-confirmed placement,
   including V1/V2/V3. Test touch use, inventory overlays, all facing cases, chest
   highlights, paste permissions and cancellation. Existing iOS core has no local
   world server; server/friend-world support must be distinguished from local worlds.
8. **Lifecycle/storage:** writes stay in Documents; bundle resources are read-only.
   Suspension cancels input/placement/mining, saves state and tolerates memory
   pressure. Preserve world/dimension scope and avoid stale placements on reconnect.
9. **Distribution:** distinct Cinnamatica app identity, bundled icon/version, unsigned
   IPA for legitimate user signing. No private profile, credentials or Mojang assets
   in public downloads. Existing user-owned asset injection remains separate.
10. **Validation/release:** focused Windows-host checks are not iOS compilation or
    acceptance. Device build requires macOS/Xcode; the user subsequently authorized GitHub Actions for an unsigned IPA; no release
    publication is implied. Record untested steps explicitly.

## References

- [Wasmtime Pulley](https://docs.wasmtime.dev/examples-pulley.html): interpreter
  target preserves Wasmtime APIs and removes the native code execution requirement.
- [Apple document picker](https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller):
  external provider access must be scoped/coordinated and copies kept in the sandbox.
- Existing `docs/ios.md`, `packaging/ios`, touch control/editor APIs and shared
  schematic decoder/placement code are the implementation sources of truth.

Status: source implemented; 109 iOS-feature guest tests and 14 shared importer tests pass on Windows. GitHub Actions arm64 iOS compilation and on-device acceptance remain.
