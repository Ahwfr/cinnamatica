# Block placement and ghost repair checklist: 0.3.63

## Evidence and scope

The user's actual client logs were inspected before editing. They distinguish
selection failures, prediction refusals and predicted state mismatches. No
private world coordinates, logs, profiles or schematics belong in this repository.

A separate stock Bedrock Dedicated Server 1.26.60.6 fixture used Survival with
allow-cheats=false. Owner console commands set up the disposable fixture and
inventory; every measured placement used normal item-use and movement packets.
Successful cases required an inventory decrement and a non-air block update,
not merely a block update packet. Each server run was capped at 210 seconds.
These measurements verify protocol rules, not the complete integrated UI path.

## Changes to check in the updated client

| Report | Identified cause and repair | In-world check |
| --- | --- | --- |
| Hoppers, cake, pot, brewing stand, campfires, hanging signs not selectable | Same-name item routes wrongly required EntityDrawn geometry; resolve any registered same-name block | Select from carried inventory and Creative catalog, then place on legitimate support |
| Berry bush, cave vines, pitcher crop unavailable | Missing seed/berry aliases; cave-vine berry forms share glow berries | Soil for berries/pitcher; underside for glow berries; growth happens later |
| Bone/hay do not place | Axis whitelist rejected their additional deprecated=0 property | All three axes, both signs of each clicked face |
| Beds fail | Fast Place's paired-item fallback admitted doors only | Foot places the pair; all four directions on a floor |
| Signs/banners do not rotate | Item resolver too narrow and yaw search had four candidates | All sixteen standing rotations and wall mounting |
| Hanging sign shape/orientation | Free hanging signs require sneak; inactive ground/facing selectors must not conflict | Free hanging under ceiling; aligned ceiling and side variants |
| Hopper orientation | Planned facing was deliberately discarded | Down outlet and each horizontal outlet with a real adjacent face |
| Bookshelf, hive, grinder, bell facing | Generic fallback used the wrong direction table | All four standing directions; grinder/bell ceiling and side attachments |
| Piston, observer, dispenser, dropper, barrel | Body-height-facing differs from yaw-facing; native predictor already models piston/dispenser/observer, barrel now does too | Horizontal from level ground; vertical only from a compatible stance |
| Crafter | Horizontal front uses the measured north/east/south/west yaw table | Four horizontal fronts; vertical/rolled variants remain limited |
| Calibrated sensor marked wrong | Fallback used opposite cardinal table; pulse phase already ignored | South/west/north/east at yaw 0/90/180/270; pulse without false state warning |
| White concrete beside grass; aquatic plants blocked by water | Several admission gates required literal air | Replace grass, and place kelp/seagrass/pickles in valid water with real support |
| Snow and pickle counts | Strict admission demanded final stack on the first item; pickles were also rejected underwater | One-at-a-time stacking, stop at guide count, pickle alive in water |
| Vines rejected | Support prediction admitted essentially stone only | Attach to loaded full solid blocks and leaves |
| Sugar cane, nether sprouts, iron door | Selection/support paths shared the above restrictions; stock real-support controls succeeded | Cane beside water on valid soil; sprouts on valid terrain; iron door on floor |
| Dirt for farmland/path | Guide asked for unobtainable soil blocks in Survival | Dirt selected automatically; yellow wrong-state guide remains until tilled/converted |
| Heads one cell too high in V2 | SkullItem offsets an air click by its face; ordinary BlockItem does not | Floating floor head at exact ghost cell; wall head uses a real face |
| Wool/concrete/prismarine placeholders | Explicit terrain stand-ins bypassed available native entity renderers | Bed/banner, every supported head, decorated pot and conduit have native shape |
| Bell body absent | Terrain only contains the frame | Body plus frame for each attachment |
| Chest/shulker brightness and outlines | Fixed native entity light/tint ignored grid instance; shulker lacked marker bounds | Brightness changes immediately; shulker closed-box outline present |
| Iron-door half invisible | Terrain mesher required another half even if hidden by verification/layers | Either missing half remains visible independently |
| Vine/lichen ghost invisible against support | Authored planes exist at 1/256 offset; reverse-Z negative slope bias could bury them | Thin planes remain visible at grazing angles; live GPU confirmation needed |

## Remaining limitations and findings

- Lily pads have no rotation property in this Bedrock registry. There is no
  block-state rotation for Fast Place to transmit.
- Stock tests rejected unsupported air clicks for conduit, pot, cake, snow and
  nether sprouts. These need legitimate support. V2 does not fabricate a server
  block or enable commands to bypass those rules.
- Nearby vertical pistons/observers/dispensers/barrels depend on eye height and
  player position. Yaw/pitch packets alone do not override that decision.
- Crafter vertical/rolled variants and side hanging-sign axes need further
  integrated coverage. Horizontal crafter direction is implemented; do not
  describe every possible crafter orientation as verified.
- Multi-face vine/lichen plans still require separate attachment uses. The
  native predictor currently resolves only the first single-face attachment;
  automatic completion of a multi-face mask is not implemented here.
- The current SCB2/grid representation carries block states, not block-entity
  NBT artwork. Native ghosts use default bed/banner artwork and plain pots.
  Floor-head NBT rotation, custom player skins, banner patterns and pot sherd
  preservation are not implemented by this update.
- All eight snow-layer heights already exist correctly in the inspected native
  asset carrier. The suspected always-one-layer rendering bug was not reproduced.
- Changed ghost shading and default native-model artwork still need visual
  confirmation on the player's graphics backend. Unit tests cannot certify this.
- The large-palette menu/fuel regression from 0.3.62 must remain covered; this
  update keeps its incremental preparation and whole retained guide design.
