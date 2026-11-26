# Cinnamatica 0.3.68: fixes and remaining checks

This update is not a claim that every registered block is obtainable or placeable
in Survival, or that every reported rendering defect has been resolved.

## Confirmed causes and implemented changes

| Report | Cause | Change / verification |
| --- | --- | --- |
| Switching away from a large schematic restarts the mod | Runtime replacement freed thousands of palette state maps in one metered WASM callback | Reproduced in 0.3.67 at callback 4022 in Schematic drop; bounded teardown passes four large/small switches at the unchanged fuel allowance |
| Windows crash in the screenshot | Pinned winit MonitorHandle::size unwrapped Win32 error 1461 after a monitor handle became invalid | Vendor the exact pinned Cinnabar winit revision with its license and mobile patches; tolerate invalid handles in name, native identifier and size |
| Glazed terracotta never places in the catalogue | Generator chose vertical facing 0; player placement generates horizontal 2/5/3/4 | Choose a valid horizontal sample; all 16 colours' four yaw results checked against production placement rules |
| Heads fail in the catalogue | Generator chose facing 0; stock server places floor heads with facing 1 | Correct the samples and floor-face handling; normal and floating skull/head item uses checked on the stock server |
| Mushroom stems always appear wrong | Catalogue used huge_mushroom_bits=0; a player places a stem with value 15 | Correct the generated stem sample; preserve requested states in real imported schematics |
| Double slabs fail | Previous path tried to substitute an equivalent full block, which is not the requested double slab | Select the real slab item, place the first slab, then allow a second use at the same cell; all 101 pinned double slab identifiers have a registered single-slab route |
| Candle cakes fail | A candle cake is not a direct inventory block | Prepare a cake, select the requested candle colour, then interact with an intact cake; ordinary and blue candle attachment accepted by stock Bedrock item-use packets |
| Inverted daylight detector fails | An inverted detector is made by interacting with a normal detector | Prepare a normal detector and use the existing controlled interaction path to invert it |
| Creaking heart cannot predict an axis | Axis resolver only accepted states containing pillar_axis alone (or deprecated) | Accept the heart's additional natural/heart-state fields without removing axis validation; all six face axes checked |
| Frames do not orient | Item-frame orientation derives from the clicked face, not yaw | Treat ordinary and glow frames as face-derived placements |
| Straw bed is half a sample and cannot use the bed placement path | Generator omitted its head; several native Fast Place checks only recognized minecraft:bed; its direction is a cardinal string | Generate its paired head; handle its paired item and cardinal heading; resolve its head back to its foot in the guest |
| Frames/chalkboard/statues render as planks or copper blocks | Hologram explicitly chose stand-in cubes | Reuse existing native frame, chalkboard and statue models; block-grid data lacks NBT, so chalkboard size/text and statue pose use defaults |
| Chests/shulkers/ender chests/conduits lack blue styling and outlines | Native entity models used a separate multiplicative tint, with no model-edge pass | Add texture-preserving wash and thin quad-perimeter outlines to the native entity material; production shader composition and Naga validation pass |
| Need configurable render distance | No user radius on the retained whole-grid path | Settings slider: 16–512 blocks; Render all: no radius limit (default). Shader clipping changes the view, not the resident geometry. Native entity pieces follow the same radius |

The generated test catalogue still covers 1,312 Minecraft identifiers, with
1,357 isolated samples and 1,406 palette states including paired pieces. Existing
sample origins are unchanged. Some samples deliberately have no support; those
are not a valid test of a support-dependent item by themselves.

## Remaining issues / limits: do not mark these fixed

| Block or problem | Evidence and remaining work |
| --- | --- |
| Upper slabs, upside-down stairs and upper trapdoors without any support | Prior stock-server tests normalize clicks into empty cells to top-face placement; fake support IDs, click heights and reported positions did not establish a working override. V1 real-support placement is preserved. No universal cheatless air-placement bypass has been demonstrated |
| Pistons, observers, dispensers, droppers and barrels in every direction from every player position | Bedrock uses the actual player's relative body position for some vertical/horizontal choices. Yaw correction is available; the prior body-position spoof tests did not override the server. Cramped-space and arbitrary-height parity remain incomplete |
| Crafter orientation | Its combined front/top orientation needs separate measured coverage; no universal fix is claimed |
| Signs in every attachment/angle | Existing four- and sixteen-direction correction remains. Catalogue wall facings are corrected; this release has not certified all sign attachment combinations |
| Floating coral blocks | Stock Survival server rejected an unsupported tube coral block; normal supported placement succeeded. Other variants still require individual support/variant checks |
| Floating beds and straw beds | Stock Survival server rejected unsupported placements. Supported straw-bed placement succeeded; a fake support packet is not established as a bypass |
| Lightning rods | A floating upward rod succeeded in the stock test. Other unsupported facings are not established; supported face-derived correction remains |
| Small dripleaf | Earlier stock checks accepted dry clay/moss and suitable submerged soil, rejected dry grass. Water alone is not a replacement for its soil/support rules; unsupported catalogue sample remains intentionally unsupported |
| Allow, deny, border, command blocks | Permission/availability rules are enforced by the server. This update does not grant restricted blocks to Survival inventories |
| Client request placeholder, frosted ice, deprecated anvil, deprecated purpur 1/2, petrified slab variants | A registered state is not proof of a usable inventory item or arbitrary player placement. Do not select a different held block as a claimed replacement; item/legacy-state coverage remains incomplete |
| Non-default mushroom cap/stem faces | Correcting the catalogue stem does not add a way to create every growth-derived mushroom face pattern by rotating a placed item |
| Closed eyeblossom and pale moss carpet shapes | Still require comparison with the native world model and pack geometry; not claimed fixed by the hologram styling change |
| Deprecated anvil cube rendering | Still follows its pinned legacy renderer; not changed to an unrelated anvil model without supporting state/model evidence |
| Far mismatches and very entity-heavy schematics | Whole textured grid stays resident, but verifier/status markers and native entity-model submissions retain resource budgets. Far wrong-state indicators and model-count overflow are not certified as unlimited |
| Separate Go core crash | A second report faults in Pion mDNS resource comparison during LAN discovery. It is separate from the Windows monitor panic; no unverified dependency upgrade or disabled friend discovery is shipped as a supposed fix |
| Java shader packs | Native Vulkan/DX12 shader programs are CPU-validated. Custom pack entity rendering may not reproduce native wash/outlines; live pack parity is not certified |

## In-game acceptance checklist

- [ ] Load test, open Materials, switch to another entry and back without recovery.
- [ ] Drag render range; choose Render all; travel between opposite corners.
- [ ] Place each glazed colour and the corrected floor heads/stem samples.
- [ ] Hold-place double slabs: both uses consume slabs from Survival inventory.
- [ ] Place ordinary and coloured candle cakes using cake plus candle.
- [ ] Place and invert a daylight detector with correction enabled.
- [ ] Place all heart axes and supported straw-bed headings, aiming at either half.
- [ ] Compare real frames, chalkboard and statues to their holograms.
- [ ] Inspect blue wash/thin model outlines on chest, shulker, ender chest and conduit.
- [ ] Re-test separately listed unfinished cases; preserve their issue status until proven.

CPU shader validation, item-route coverage and WASM callback tests do not certify
live GPU appearance or integrated multiplayer placement. No installer publication
or GitHub Actions run is part of this local update.
