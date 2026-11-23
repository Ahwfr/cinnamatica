# Large-palette WASM regression probe

## Native placement rule probe

`placement-rules.rs` directly includes the owning gameplay placement modules and
item-route module. Link it against this checkout's `assets`, `serde_json` and
`serde` rlibs using the same Rust toolchain and dependency directory as the client.
It is a small executable, not an app/UI test build. With an abort-panic client
profile, compile it with `-C panic=abort` too.

It checks real pinned-registry item aliases, all six bone/hay click faces, the
stock-server-measured sensor/campfire/bookshelf/hive yaw tables, and horizontal
and body-height barrel facing. The four connection constants are supplied only
to satisfy the included placement-state module; fence/pane prediction is not
tested by this probe. It completed successfully for 0.3.63. It does not send
packets or certify the integrated Fast Place UI.

## Large palettes

`large-palette-budget.rs` drives the real guest through `mod_host`, using its
normal 1,000,000 fuel frame limit. Native guest unit tests cannot detect fuel traps.

Compile this standalone probe against the matching checkout's `mod_host` and
`mod_api` rlibs (and dependency/native-library directories) with its pinned Rust
toolchain. Then run it with a component path as its single argument. Do not
increase the frame allowance to make the test pass.

The fixture must contain one schematic in library slot 0, at least 10,000
bounding-box cells and at least 800 unique required item identifiers. It is a
private/local input, not a published fixture. Build the component with the
current importer `library OUTPUT.wasm INPUT.litematic` command and matching
`SCHEMATICA_GUEST_TEMPLATE`. Choose a fresh output filename.

The probe checks loading, full-grid submission, native geometry-ready feedback,
container highlight publication, populated Materials rows, and continued
verification after closing Materials. It simulates native mesh completion;
it does not replace an actual GPU or multiplayer acceptance check.

For 0.3.62 the reported 23101 input was tested (67x5x62, 853 palette states,
816 materials). The 0.3.61 component exhausted fuel in
`Viewer::publish_item_highlights`; after that was fixed, Materials exhausted fuel
while serializing the panel after preparing its rows. The updated probe completes
more than 4,000 callbacks with the same per-frame allowance.
