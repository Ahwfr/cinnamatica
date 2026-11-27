# Cinnamatica 0.3.69: evidence and remaining checks

## Fixed in this update

- Imported and already-saved double slabs normalize to full blocks before palette counting. The generated table covers all 101 registered double-slab types; `verification/full-block-equivalents.rs` verifies registered full-block equivalents. Materials, previews, verification and paste consume the same normalized schematic. This intentionally changes double-slab semantics as requested; original source files are preserved.
- Eyeblossoms were classified as unknown six-face models. The pack compiler now admits unknown, flat-icon flower shapes through its existing crossed-plant renderer. Recompiled carriers contain two crossed quads for both open and closed eyeblossoms.
- The test generator no longer invents an upper pale-moss carpet with no side connections. This repairs the generated test; it is not a claim that every imported connected moss state was changed.
- Independent guide geometry uses a world anchor. A block anchor was dependent on its origin chunk staying loaded, causing the whole retained guide to disappear when that corner unloaded. Native beds, chests and other entity-model pieces also accept integral world anchors.
- An inverted daylight detector uses the obtainable normal sensor, but stays WrongState/yellow until manually inverted. Fast Place no longer automatically toggles it.
- Split tests into Working baseline (1,170 samples), Known issues (65), and Unverified fixes (122), grouped by block family. Working means absent from the latest failure report, not proof of every possible orientation or unsupported placement. All 1,357 source samples remain represented. The combined catalogue and JSON coordinates are exported too.
- Library capacity is 16 entries with pages of eight, sharing the unchanged embedded byte budget. Existing library indices, deletion flags and placement revision can be preserved when installing the split tests. SCB exports can be imported directly on Windows and Linux.

- Add measured native crafter prediction: all 12 orientation states match captured stock-server placements. Barrel and crafter standing hints include body-height requirements. An optional export provides 18 isolated orientation stations; normal catalogue samples start horizontal.
- Adapt native banner dye inputs to upstream gamma-space colors, keeping ghost wash words out of actor lighting. Owning mesh-emitter verification covers dye conversion, retained wash/outline flags and quad coordinates.

## Confirmed server limits / unresolved cases

A local Bedrock Dedicated Server 1.26.60.6 probe used Survival with cheats disabled, normal inventory item uses, and consistent Pitch, Yaw, HeadYaw, InteractPitch, InteractYaw and CameraOrientation. Console commands only established the isolated test fixtures and inventory.

- Barrel, crafter, piston, dispenser, dropper and observer accepted unsupported horizontal placement. Changing all angle fields between -89, 0 and 89 degrees from the same body position did not produce arbitrary vertical orientations. No universal orientation bypass was established. Embedded PlayerAuthInput item interactions and client item-runtime/metadata echoes also did not override this behavior.
- Conduit and coral accepted supported placement but rejected the empty-cell transaction; inventory was not consumed. Do not label unconditional air placement fixed.
- Lightning rods accepted upward air placement. The downward catalogue state still needs a suitable support face; it must not silently place the wrong state.
- Candle cakes remain a two-use operation (cake, then matching candle) and need a supporting surface for the base cake. The missing native LocalUse classification is fixed for all 17 candles: an intact cake is an in-cell interaction, not an adjacent placement. Bitten/unreadable cakes, empty hands and denied building retain refusal. The owning classifier passes a fast real-registry probe; the complete native Cargo test compilation hit its four-minute cap and is not counted as passing. Live game confirmation remains necessary.
- Distant Wrong/WrongState cells still rely on the local status-marker window for their colored marker. This is separate from origin-chunk disappearance and remains a rendering limitation.

## Upstream update plan

The installed 0.3.68 source was a38ce3b, based on upstream d2195d6 (Cinnabar package 0.2.1 / Bedrock 1.26.60 protocol 2225). At audit time upstream HEAD is 8ae394f50, 22 commits ahead. Latest stable tag remains v0.2.1; this update targets latest upstream source, not a nonexistent newer release.

Changes include idle block-entity and cave-visibility performance, sky draw order, block-entity lighting, head placement, inventory armor prediction, camera/perspective API, browser rendering and spectator work, Adventure edit markers, eating/use arbitration, and Windows WiX packaging. Preserve the fork's guide grid pipeline, placement and inventory APIs, native ghost models, shaders, model/compiler fixes, installer branding, and Linux contributions. Merge in the isolated checkout, resolve overlaps explicitly, rebuild native helpers and guest, exercise focused placement/render/API checks under four-minute test caps, then replace the closed local launcher.

## Not verified in game

This build compiled; most placement behavior has not been exercised live. In particular unverified: V3 supports for signs, vines, torches, buttons, banners, candles, eggs, lichen, beds and pillars; the soil check and creative soil placement; wall and torch support side; the cleanup of temporary supports in Survival (reported not breaking them); the walking pull-back while placing (server corrections were seen in a creative-flying session, cause unconfirmed); the Settings tab with the new highlight switch; hidden red wrong-block boxes behind flowers; sometimes dropping an item picked from the creative inventory.

## In-game checklist

- Load an old double-slab schematic: materials and ghosts should show full blocks.
- Load each of the three test sets; switch pages and load/delete a last-page entry.
- With Render all, move far enough to unload the origin chunk; ordinary and native-model ghosts should remain visible.
- Check both eyeblossom textures and the flat pale moss sample.
- Place a sensor for an inverted guide: yellow before use, correct after manual inversion.
- Retry candle cake on an actual supporting block; report the native log if it still refuses.
- Recheck existing hold placement, menus, materials, paste and saved world placements.

## Inventory-backed support mode and current UI

V3 Support is separate from V1/V2. It uses real inventory blocks and real server updates, not fake support runtime IDs or commands. Intended schematic support cells take priority; otherwise a bounded preference ranks dirt/coarse dirt, netherrack, common stone variants and planks, with abundance and hotbar tie-breaks. An empty eligible inventory reports a local message. Preparation is bounded to two support levels and loaded cells within reach. Occupied cells are never cleared by this mode. Supports needed for falling blocks, repeaters, candles and similar ground-dependent targets remain. Expendable blocks are removed only if their exact confirmed receipt and current runtime identity still match, through native timed mining and its ordinary reach/tool/game-mode checks. Cancellation, changed terrain or missing confirmation may leave a support in place. This is intentionally conservative; it must never delete an unrelated block.

Materials now counts at most sixteen matching palette entries at a time rather than allowing one large material/tile group to exhaust a callback. The layer scope builds thirty-two cells per callback, independently of the render range. Existing logs confirmed frame-fuel exhaustion on opening Materials. Guest unit tests cover bounded progress; stock-server acceptance of the complete controller and metered large-file regression must be recorded separately before claiming live success.

Orientation is always enabled for Easy Place; the redundant orientation/nearby-assist switches are retired. The Materials scope button and precise lightness slider save immediately. Controls groups movement by view direction, rotation/guide state, layers, and Paste. Remove wrong blocks belongs to Paste and preserves correct blocks. V1/V2 repeat admission uses the physics tick instead of an additional wall-clock delay. Already-written orientations can be reused; new angles and inventory changes still require normal admission. Movement input vectors and directional flags are transformed into the temporary reported-facing frame while physical movement and the visible camera stay unchanged.
