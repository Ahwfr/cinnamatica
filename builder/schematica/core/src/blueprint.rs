use crate::{BlockState, Schematic, StateValue};

pub const MAX_EDGE: i32 = 128;
pub const MAX_CELLS: usize = 400_000;
// Enough for the complete current Bedrock block catalogue plus orientation/color
// variants. Palette data still shares the bounded embedded-library byte budget.
pub const MAX_PALETTE: usize = 4096;
pub const SLOT_MARKER: &[u8] = b"schematica:embedded-library:v4\x00";
pub const MAX_LIBRARY: usize = 8;
// Preserve the existing fixed WASM slot and old imported libraries. The 32 KiB term
// describes the original slot allocation, not a limit on any decoded palette.
const LIBRARY_BYTES: usize = 13 + MAX_LIBRARY * (4 + 200_000 + 32 * 1024);
// One SCL2 header and entry length precede a single entry. Palette and cell data
// share the available capacity instead of imposing a separate palette-byte cap.
const PAYLOAD_BYTES: usize = LIBRARY_BYTES - 13 - 4;
pub const SLOT_BYTES: usize = SLOT_MARKER.len() + 4 + LIBRARY_BYTES;

pub const fn empty_slot() -> [u8; SLOT_BYTES] {
    let mut slot = [0; SLOT_BYTES];
    let mut index = 0;
    while index < SLOT_MARKER.len() {
        slot[index] = SLOT_MARKER[index];
        index += 1;
    }
    slot
}

/// Encodes independent blueprints so the guest can decode only the selected one.
pub fn encode_library(blueprints: &[Vec<u8>]) -> Result<Vec<u8>, String> {
    if !(1..=MAX_LIBRARY).contains(&blueprints.len()) {
        return Err(format!("choose between 1 and {MAX_LIBRARY} schematics"));
    }
    let mut bytes = b"SCL1".to_vec();
    bytes.push(blueprints.len() as u8);
    for payload in blueprints {
        if payload.len() > PAYLOAD_BYTES {
            return Err("blueprint exceeds the library entry limit".into());
        }
        bytes.extend((payload.len() as u32).to_le_bytes());
        bytes.extend(payload);
    }
    if bytes.len() + 8 > LIBRARY_BYTES {
        return Err("schematic library is full by size; remove an entry before importing".into());
    }
    Ok(bytes)
}

/// A fresh import identity lets users restore deleted entries by importing again.
pub fn encode_library_revision(blueprints: &[Vec<u8>], revision: u64) -> Result<Vec<u8>, String> {
    let original = encode_library(blueprints)?;
    let mut bytes = b"SCL2".to_vec();
    bytes.extend(revision.to_le_bytes());
    bytes.extend_from_slice(&original[4..]);
    Ok(bytes)
}

pub fn library_revision(bytes: &[u8]) -> u64 {
    if bytes.starts_with(b"SCL2") && bytes.len() >= 12 {
        u64::from_le_bytes(bytes[4..12].try_into().unwrap())
    } else {
        bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
    }
}

pub fn library(mut bytes: &[u8]) -> Result<Vec<&[u8]>, String> {
    if bytes.len() > LIBRARY_BYTES {
        return Err("encoded schematic library exceeds its storage capacity".into());
    }
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.starts_with(b"SCB1") || bytes.starts_with(b"SCB2") {
        return Ok(vec![bytes]);
    }
    match take(&mut bytes, 4)? {
        b"SCL1" => {}
        b"SCL2" => {
            take(&mut bytes, 8)?;
        }
        _ => return Err("invalid blueprint library".into()),
    }
    let count = byte(&mut bytes)? as usize;
    if !(1..=MAX_LIBRARY).contains(&count) {
        return Err("invalid blueprint count".into());
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let len = u32::from_le_bytes(take(&mut bytes, 4)?.try_into().unwrap()) as usize;
        if len > PAYLOAD_BYTES {
            return Err("blueprint exceeds the library entry limit".into());
        }
        entries.push(take(&mut bytes, len)?);
    }
    if !bytes.is_empty() {
        return Err("unexpected data after blueprint library".into());
    }
    Ok(entries)
}

pub fn title(bytes: &[u8]) -> Result<String, String> {
    Ok(Decoder::new(bytes)?.name)
}

/// The blueprint's size from its header, without decoding its cells.
pub fn size(bytes: &[u8]) -> Result<[i32; 3], String> {
    Ok(Decoder::new(bytes)?.size)
}

/// Builds a fixed-size data slot that the desktop converter can populate without recompiling.
pub fn embedded_slot(payload: &[u8]) -> Result<Vec<u8>, String> {
    if payload.len() > LIBRARY_BYTES {
        return Err("blueprint exceeds the embedded slot".into());
    }
    let mut slot = SLOT_MARKER.to_vec();
    slot.extend((payload.len() as u32).to_le_bytes());
    slot.extend_from_slice(payload);
    slot.resize(SLOT_BYTES, 0);
    Ok(slot)
}

pub fn embedded_payload(slot: &[u8]) -> Option<&[u8]> {
    let header = slot.strip_prefix(SLOT_MARKER)?;
    let (len, bytes) = header.split_at_checked(4)?;
    bytes.get(..u32::from_le_bytes(len.try_into().ok()?) as usize)
}

pub fn validate_size(size: [i32; 3]) -> Result<usize, String> {
    if !size.iter().all(|n| (1..=MAX_EDGE).contains(n)) {
        return Err(format!("each dimension must be 1–{MAX_EDGE}, got {size:?}"));
    }
    let cells = size.iter().product::<i32>() as usize;
    if cells > MAX_CELLS {
        return Err(format!(
            "bounding box has {cells} cells, limit is {MAX_CELLS}"
        ));
    }
    Ok(cells)
}

pub fn encode(schematic: &Schematic) -> Result<Vec<u8>, String> {
    let [sx, sy, sz] = schematic.size();
    let cells = validate_size([sx, sy, sz])?;
    if schematic.palette().len() > MAX_PALETTE {
        return Err(format!(
            "palette exceeds {MAX_PALETTE} distinct block states"
        ));
    }
    let mut out = b"SCB2".to_vec();
    out.extend([sx as u8, sy as u8, sz as u8]);
    out.extend((schematic.palette().len() as u16).to_le_bytes());
    put_str(&mut out, &schematic.name)?;
    for block in schematic.palette() {
        put_str(&mut out, &block.name)?;
        out.push(u8::try_from(block.states.len()).map_err(|_| "too many block properties")?);
        for (name, value) in &block.states {
            put_str(&mut out, name)?;
            match value {
                StateValue::Bool(v) => out.extend([0, u8::from(*v)]),
                StateValue::Int(v) => {
                    out.push(1);
                    out.extend(v.to_le_bytes());
                }
                StateValue::Str(v) => {
                    out.push(2);
                    put_str(&mut out, v)?;
                }
                StateValue::Byte(v) => out.extend([3, *v as u8]),
            }
        }
    }
    let encoded_bytes = out.len() + 2 * cells;
    if encoded_bytes > PAYLOAD_BYTES {
        return Err(format!(
            "decoded schematic needs {encoded_bytes} bytes ({} bytes of block states); \
             library-entry capacity is {PAYLOAD_BYTES} bytes. Compressed file size is different",
            out.len()
        ));
    }
    for y in 0..sy {
        for z in 0..sz {
            for x in 0..sx {
                let index = schematic.palette_index([x, y, z]).unwrap() as u16;
                out.extend(index.to_le_bytes());
            }
        }
    }
    Ok(out)
}

fn put_str(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    out.push(u8::try_from(value.len()).map_err(|_| "string exceeds 255 bytes")?);
    out.extend(value.as_bytes());
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<Schematic, String> {
    let mut decoder = Decoder::new(bytes)?;
    while !decoder.step()? {}
    decoder.finish().map(|(schematic, _)| schematic)
}

/// Incrementally decodes one palette entry at a time before copying the bounded cell array.
pub struct Decoder<'a> {
    bytes: &'a [u8],
    size: [i32; 3],
    count: usize,
    volume: usize,
    wide: bool,
    name: String,
    palette: Vec<BlockState>,
    blocks: Vec<u32>,
    counts: Vec<u64>,
}

impl<'a> Decoder<'a> {
    pub fn new(mut bytes: &'a [u8]) -> Result<Self, String> {
        if bytes.len() > PAYLOAD_BYTES {
            return Err("decoded schematic exceeds the library-entry capacity".into());
        }
        let wide = match take(&mut bytes, 4)? {
            b"SCB1" => false,
            b"SCB2" => true,
            _ => return Err("invalid blueprint signature".into()),
        };
        let size = [
            byte(&mut bytes)? as i32,
            byte(&mut bytes)? as i32,
            byte(&mut bytes)? as i32,
        ];
        let volume = validate_size(size)?;
        let count = if wide {
            u16::from_le_bytes(take(&mut bytes, 2)?.try_into().unwrap()) as usize
        } else {
            byte(&mut bytes)? as usize
        };
        if !(1..=MAX_PALETTE).contains(&count) {
            return Err("invalid palette size".into());
        }
        let name = string(&mut bytes)?;
        Ok(Self {
            bytes,
            size,
            count,
            volume,
            wide,
            name,
            palette: Vec::with_capacity(count),
            blocks: Vec::new(),
            counts: vec![0; count],
        })
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.palette.len() == self.count {
            for _ in self.blocks.len()..(self.blocks.len() + 512).min(self.volume) {
                let index = if self.wide {
                    u16::from_le_bytes(take(&mut self.bytes, 2)?.try_into().unwrap()) as usize
                } else {
                    byte(&mut self.bytes)? as usize
                };
                let count = self
                    .counts
                    .get_mut(index)
                    .ok_or("palette index out of range")?;
                *count += 1;
                self.blocks.push(index as u32);
            }
            if self.blocks.len() == self.volume {
                return if self.bytes.is_empty() {
                    Ok(true)
                } else {
                    Err("unexpected blueprint data".into())
                };
            }
            return Ok(false);
        }
        let bytes = &mut self.bytes;
        let mut state = BlockState::new(string(bytes)?);
        for _ in 0..byte(bytes)? {
            let key = string(bytes)?;
            let value = match byte(bytes)? {
                0 => StateValue::Bool(byte(bytes)? != 0),
                1 => StateValue::Int(i32::from_le_bytes(take(bytes, 4)?.try_into().unwrap())),
                2 => StateValue::Str(string(bytes)?),
                3 => StateValue::Byte(byte(bytes)? as i8),
                _ => return Err("unknown state value type".into()),
            };
            state.states.insert(key, value);
        }
        self.palette.push(state);
        Ok(false)
    }

    pub fn finish(self) -> Result<(Schematic, Vec<u64>), String> {
        if self.palette.len() != self.count
            || self.blocks.len() != self.volume
            || !self.bytes.is_empty()
        {
            return Err("blueprint has not finished decoding".into());
        }
        Ok((
            Schematic::from_validated(self.name, self.size, self.palette, self.blocks),
            self.counts,
        ))
    }
}

fn take<'a>(bytes: &mut &'a [u8], size: usize) -> Result<&'a [u8], String> {
    let (prefix, rest) = bytes.split_at_checked(size).ok_or("truncated blueprint")?;
    *bytes = rest;
    Ok(prefix)
}

fn byte(bytes: &mut &[u8]) -> Result<u8, String> {
    Ok(take(bytes, 1)?[0])
}

fn string(bytes: &mut &[u8]) -> Result<String, String> {
    let len = byte(bytes)? as usize;
    String::from_utf8(take(bytes, len)?.to_vec()).map_err(|_| "invalid UTF-8".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_legacy_blueprints() {
        let legacy = b"SCB1\x01\x01\x01\x01\x01A\x0dminecraft:air\x00\x00";
        assert_eq!(decode(legacy).unwrap().name, "A");
    }

    #[test]
    fn wide_palette_and_cardiff_sized_volume_round_trip() {
        let mut schematic = Schematic::new("large", [47, 45, 36]);
        for index in 0..300 {
            schematic.set(
                [index % 47, 0, index / 47],
                BlockState::new(format!("test:block_{index}")),
            );
        }
        let bytes = encode(&schematic).unwrap();
        let restored = decode(&bytes).unwrap();
        assert_eq!(restored.palette().len(), 301);
        assert_eq!(restored.palette_index([17, 0, 6]), Some(300));
        assert!(decode(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn large_decoded_palette_round_trips_through_the_existing_library_slot() {
        let mut schematic = Schematic::new("many states", [100, 1, 5]);
        for index in 0..500 {
            schematic.set(
                [index % 100, 0, index / 100],
                BlockState::new("minecraft:test")
                    .with("variant", StateValue::Int(index))
                    .with(
                        "description",
                        StateValue::Str("repeated state data ".repeat(8)),
                    ),
            );
        }
        let payload = encode(&schematic).unwrap();
        assert!(payload.len() - 2 * 500 > 32 * 1024);
        let encoded = encode_library_revision(&[payload.clone()], 61).unwrap();
        let slot = embedded_slot(&encoded).unwrap();
        assert_eq!(slot.len(), SLOT_BYTES);
        let entries = library(embedded_payload(&slot).unwrap()).unwrap();
        let restored = decode(entries[0]).unwrap();
        assert_eq!(restored.palette(), schematic.palette());
        assert_eq!(
            restored.layer_cells(crate::LayerMode::All),
            schematic.layer_cells(crate::LayerMode::All)
        );
    }

    #[test]
    fn palette_and_cells_share_the_total_entry_capacity() {
        let mut schematic = Schematic::new("too many state bytes", [64, 1, 1]);
        for index in 0..64 {
            let mut state = BlockState::new(format!("minecraft:test_{index}"));
            for property in 0..128 {
                state = state.with(
                    &format!("property_{property}"),
                    StateValue::Str("x".repeat(240)),
                );
            }
            schematic.set([index, 0, 0], state);
        }
        assert!(encode(&schematic)
            .unwrap_err()
            .contains("library-entry capacity"));
        assert!(Decoder::new(&vec![0; PAYLOAD_BYTES + 1]).is_err());
        assert!(library(&vec![0; LIBRARY_BYTES + 1]).is_err());
    }

    #[test]
    fn extended_palette_round_trips_and_rejects_the_next_entry() {
        let mut schematic = Schematic::new("full catalogue", [64, 1, 64]);
        for index in 1..MAX_PALETTE {
            schematic.set(
                [(index % 64) as i32, 0, (index / 64) as i32],
                BlockState::new("minecraft:test").with("variant", StateValue::Int(index as i32)),
            );
        }
        let bytes = encode(&schematic).unwrap();
        assert_eq!(decode(&bytes).unwrap().palette(), schematic.palette());
        // Cell zero was air; adding a new state exceeds the palette bound.
        schematic.set([0, 0, 0], BlockState::new("minecraft:over_limit"));
        assert!(encode(&schematic)
            .unwrap_err()
            .contains("distinct block states"));
        let mut invalid = bytes;
        invalid[7..9].copy_from_slice(&((MAX_PALETTE + 1) as u16).to_le_bytes());
        assert!(Decoder::new(&invalid).is_err());
    }
}
