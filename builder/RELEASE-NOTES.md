# Cinnamatica 0.3.64

- Resolve bed-head placement targets to their paired foot through schematic rotations and mirrors, and solve bed placement against a floor instead of an existing bed side.
- Exclude replaceable vegetation and fluids from adjacent support candidates; keep direct replacement at the requested destination.
- Broaden snow-layer support prediction to measured solid surfaces while excluding ice, frosted ice and barriers. Predict small dripleaf pairs on dry clay/moss or suitable submerged soil.
- Preserve requested hotbar selection during pending Fast Place preparation and log concrete cancellation reasons for further diagnosis.
- Preserve dye colors on newly imported Java beds and banners, render their corresponding native artwork, and select the matching inventory variant. Previously imported color data cannot be reconstructed; reimport the original schematic. Pattern artwork and block-entity color verification/pasting are not added by this change.
- Correct decorated-pot facing, floating grindstone yaw correction and dragon-head outline bounds.
- See BLOCK-BUGS-0.3.64.md for measured server constraints and the testing checklist. This does not introduce commands into Fast Place or promise every unsupported orientation is accepted by the server.

# Cinnamatica 0.3.63

- Preserve the newer public-main Debug target readout, session-derived item routes, dropper prediction and stateless-block placement fixes (through public commit 460199c6d), alongside this repair.

- Resolve registered item-form blocks even when their inventory stack advertises no block runtime ID. Add checked glow-berry, sweet-berry and pitcher-pod routes.
- Correct sixteen-step standing sign/banner rotation, bed admission, hopper outlets, hanging-sign sneak selection, and the measured facing rules for bookshelves, grindstones, bells, beehives, campfires and calibrated sensors. Preserve bone/hay axis properties.
- Allow replaceable vegetation and water destinations. Build snow/pickle/candle stacks one item at a time. In Survival, offer dirt for farmland/path guides and label that interim soil as wrong state.
- Render beds, banners, heads, pots, conduits and bell bodies using native entity models. Entity ghosts now follow grid brightness/tint; add shulker outlines. Keep independently visible door halves meshed and remove slope depth bias that can bury thin vine/lichen planes behind real supports.
- Correct the measured one-cell skull offset for V2 air clicks. Keep server-required support constraints; a packet cannot make every attachment legal in empty air.
- See `BLOCK-BUGS-0.3.63.md` for evidence, remaining limitations and the in-world checklist. Native compilation, the production-module registry/rule probe and 104 guest tests passed during development; live GPU appearance and the complete integrated placement matrix still require testing with this client.

# Cinnamatica 0.3.62

- Fix guest restarts while loading schematics with large palettes. Container-highlight identifiers and old-list cleanup are processed incrementally, with publication on a separate frame, instead of sorting hundreds of identifiers after counting the guide.
- Bound Materials name preparation and palette matching so panel serialization has enough fuel. Large lists finish preparing without restarting the mod or closing the menu.
- Keep the whole retained hologram renderer and existing imported library format unchanged.
- Verified with the reported 23101 schematic in the actual WASM host at its normal frame limit: full grid submitted, more than 800 highlight identifiers published, Materials rows populated, and over 4,000 callbacks without recovery. The probe simulates native mesh-ready feedback; live GPU appearance is not verified by it. Also covered by 97 guest unit tests, including a 1,024-material preparation regression.

# Cinnamatica 0.3.61

- Remove the independent 32 KiB decoded-palette limit. Block-state descriptions and cell data now share the existing library-entry capacity, in both the importer and guest decoder. The fixed library slot and existing imported libraries stay compatible.
- Raise the importer's input and gzip-decoded byte allowance from 8 MiB to 64 MiB. These are distinct from encoded palette size and schematic volume.
- Keep bounded library/component storage, 1,024 palette states, 128-block dimensions and 400,000 bounding-box cells. This is not unlimited Litematica-size support; Litematica's practical memory limits are not equivalent to this client's fixed embedded library.

# Cinnamatica 0.3.60

Everything from 0.3.59, plus:

- **Placed and broken blocks reach the hologram sooner.** The mod gets 1,000,000 fuel a frame, and the check that notices a block placed or broken near you used to run late in the frame, after Fast Place and the section checks had spent part of it. It was skipped whenever less than 650,000 was left, and the ghost's update at the end of the frame needed 400,000 more, so on a busy frame a change simply waited. Now:
  - The check runs **first** in the frame, so it is never starved, and Fast Place decides on what the world holds now, not on what it held a few frames ago.
  - Its comparison is far cheaper. It used to rebuild a block id for every one of the 4,096 cells it watches, then compare them one by one; it now compares whole runs of 64 cells with one test each and looks closer only at the runs that differ. The same goes for the cells that both the old and the new box hold when you walk and the box moves. A test checks it against the old cell-by-cell answer over many random changes, unloaded cells, palette changes and box moves.
  - The lookup that decides what a cell's block means no longer uses the standard hash, which cost several times more fuel than the lookup itself.
  - The ghost's edits go to the game right after the check queues them, not at the end of the frame, and the end-of-frame send needs 300,000 left instead of 400,000. The outline redraw needs 500,000 instead of 700,000.

Verified: the mod's unit tests (96), the mod's real WASM build compiles (checked with a separate Rust 1.100 toolchain, which also now catches errors that only show in the WASM build). The full client was not built here. Not tried in the game: everything above.

---

# Cinnamatica 0.3.59

Everything from 0.3.57, plus:

- **A double chest gets one outline round both halves**, not one frame per half. Two halves of the same kind facing the same way, side by side, share one box while both are still missing. If one half is already placed, the other keeps its own frame.
- **The section check cannot be starved.** 0.3.57 made it wait for 1.5 times what a check cost. A mod that is always busy might never reach that, so sections would never be re-checked. After 24 frames in a row where it was held back, it uses the old fixed limit for one check.

Verified: the mod's unit tests (93, with new ones for the double chest frame and the fallback gate). The mod's WASM file and the full client were not built here. Not tried in the game: everything above.

