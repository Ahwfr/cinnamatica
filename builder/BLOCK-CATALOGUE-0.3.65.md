# Complete block catalogue and expanded palettes

The importer and guest now accept up to 4,096 palette states per schematic instead
of 1,024. The existing volume, decoded-byte, embedded-library and native drawing
limits still apply. A palette entry is a complete block state; rotations can use
multiple entries for the same block identifier. Existing SCB/SCL files remain valid.

Large native-grid transfer palettes are cloned and retired in bounded batches.
The native renderer still receives one complete retained grid: this does not turn
the hologram into separately appearing or periodically refreshed pieces.

Validation: boundary codec round-trip/rejection tests; a real-WASM normal-frame-fuel
regression using a generated 1,313-identifier/1,406-state catalogue, including the
Materials panel and thousands of callbacks without guest recovery. Native mesh
completion is simulated in this regression; it is not a live GPU acceptance test.

The generated local schematic is named `test`. It contains 1,358 separated samples,
including all 16 bed colors and standing/wall banner colors, in a 120 x 4 x 102
volume. Origins are on a three-block grid. There are no shared floors, ceilings or
supports; the six-direction spacing check permits adjacency only inside a single
paired sample (for example the two halves of a bed or door). The generator checks
that every identifier in the pinned protocol-2225 registry appears in the palette.

Invisible/technical blocks and air are deliberately included. Being registered
does not mean a block can be obtained or placed in Survival, or that the renderer
will display an otherwise invisible block. This catalogue covers every registered
block identifier, not all 23,960 property combinations or every block-entity NBT design.

This is a guest/importer update using the unchanged 0.3.64 native runtime. No
installer publication is part of this update.
