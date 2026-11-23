# Placement investigation and 0.3.64 checklist

## Confirmed client defects repaired

- Bed heads were explicitly excluded as placement targets. They now target their paired foot, including transformed schematics. Beds use paired floor admission rather than an existing bed's side.
- Bridge support search accepted replaceable vegetation. Native destination selection then replaced that vegetation, producing the one-cell offset seen in the logs. Adjacent anchors now exclude air, replaceable vegetation and fluids; direct replacement in the planned cell remains allowed.
- Snow prediction accepted almost only stone/leaves. The stock server accepted wood, grass, glass, clay, moss, packed ice and blue ice. Ordinary ice, frosted ice and barrier rejected snow and remain excluded.
- Small dripleaf had no matching two-cell predictor. The new predictor handles measured dry clay/moss and submerged soil. Water alone without an admissible substrate does not suffice.
- A pending selection could be cancelled because the selected slot had changed to the requested slot. That expected transition is now accepted. Other target/movement/input guards remain, with cancellation reasons logged.
- Java-to-Bedrock conversion discarded bed/banner colors. New imports retain a bounded dye hint, native ghosts render it, and inventory selection matches its auxiliary value. This hint is excluded from server block-state comparison and paste properties. Existing imported files need reimporting; banner patterns and colored block-entity paste/verification remain separate work.
- Decorated-pot native models did not read their direction property. Floating standing grindstones could lose yaw correction when no real support was found. Dragon heads used the ordinary skull's tiny outline inside the model; bounds now enclose the measured native model.

## Server experiments, not a universal bypass

Disposable stock Bedrock Dedicated Server 1.26.60.6 worlds used Survival with cheats disabled. Console fixture setup provided items and substrates; placement used item transactions and auth input, not commands. Success required inventory decrement and the returned non-air block state, not merely an update packet.

- Forging the item transaction's player position did not change piston, observer, dispenser, dropper or barrel orientation selected from the actual stance.
- Forging auth-input positions caused server movement corrections and did not produce the desired orientation.
- Unsupported air clicks on multiple faces and high/out-of-cell click heights still produced bottom slabs, upright stairs and lower trapdoors.
- Advertising a stone runtime ID for an air cell was rejected rather than creating a server-side support block.
- Beds and doors placed on a real floor, but fully unsupported variants were rejected.
- Hanging signs placed against a real ceiling/side, with different attached/rotation properties depending on sneak. This does not establish arbitrary unsupported sign placement.

No reliable universal orientation/no-support bypass was found in this matrix. These repairs preserve both existing Fast Place modes rather than pretending rejected packets succeeded.

Protocol references: [item-use transaction](https://mojang.github.io/bedrock-protocol-docs/1.26.60/types/item-use-inventory-transaction/), [auth input](https://mojang.github.io/bedrock-protocol-docs/1.26.60/packets/player-auth-input-packet/), [authoritative movement](https://github.com/Mojang/bedrock-protocol-docs/blob/main/additional_docs/PlayerMovementOverview.md). Archived Horion scaffold searches actual neighbouring blocks; it is not evidence of a modern universal virtual-support bypass.

## In-world checklist

- [ ] Aim at either bed half in a rotated/mirrored schematic with both floor cells present.
- [ ] Place wooden/iron doors on a floor, with Placement Lock on and off.
- [ ] Place concrete beside grass; separately replace grass occupying the desired cell.
- [ ] Snow on wood/grass/glass/clay; reject ordinary ice.
- [ ] Small dripleaf on dry moss/clay and submerged grass/clay.
- [ ] Hanging signs against ceiling and side, including attached/sneak variants.
- [ ] Decorated pots and standing grindstones in all horizontal directions.
- [ ] Reimport colored beds/banners and check ghost color and selected item.
- [ ] Dragon-head outline from several viewing angles.
- [ ] Regression: normal solid blocks, inventory swapping, hold placement and large Materials panel.

Native compilation and focused probes cannot substitute for this integrated gameplay checklist. Elevation-dependent orientations and unsupported upper slabs/stairs/trapdoors remain unresolved; do not mark them completed based on the above client repairs.
