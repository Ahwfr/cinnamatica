# Complete-gallery rendering repair

The generated gallery accidentally included `cinnabar:reserved`, an internal
registry placeholder rather than a Minecraft block. The live named-state lookup
deliberately excludes it. Resolving the entire retained grid therefore returned
`Invalid: block state`; the host reported `mesh Invalid: block renderer`, and the
guest switched to its nearby verifier-window fallback. This explains why ordinary
blocks disappeared with distance as well as entity-model blocks.

The earlier real-WASM test simulated native mesh completion, and the earlier CPU
mesh probe used a permissive raw-record lookup. Neither reproduced that exclusion.
The new `verification/block-catalogue-mesh.rs` uses the same
`PhysicsCollisionRegistries` lookup and default-state fallback as the client worker.
The archived bad catalogue fails on the placeholder in both sequential and hashed
modes. The corrected catalogue completes its full native CPU mesh in both modes.
These probes still do not replace live GPU verification.

The corrected `test` covers all **1,312 Minecraft identifiers**, with 1,357 samples
including bed/banner dyes and 1,405 schematic palette states including paired
pieces. It excludes all non-`minecraft:` implementation placeholders. Every real
sample keeps its original local position: the former placeholder station is empty.
The 120 x 4 x 102 dimensions, isolated spacing, library index and saved placement
revision remain unchanged. Other imported schematics are not modified.

A separate hard 64-block rejection in native ghost entity-model collection was
removed. Beds, chests, banners and heads no longer have that additional distance
limit. Normal camera visibility, depth testing, existing per-frame/per-grid model
limits and GPU memory limits remain in effect. This requires the updated native
client as well as the corrected schematic; Fast Place and paste logic are unchanged.
