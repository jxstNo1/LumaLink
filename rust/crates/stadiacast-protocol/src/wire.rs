//! WireHeader2 framing, ported from `src/stadiacast/rtc/wire.py`.
//! Copyright (c) 2026 Francisco Jimenez Cabrera and the Stadiacast contributors.
//! Licensed under the MIT License; see the repository's LICENSE.

use std::fmt;

pub const HEADER_LEN: usize = 8;
pub const MAX_TOTAL_LEN: usize = 0xFF_FFFF;
pub const ID_INPUT: u16 = 0x66;
pub const ID_GAME_RPC: u16 = 0xFFFA;
pub const ID_CONTROLLER_RPC: u16 = 0x74;
pub const ID_DISPLAY_RPC: u16 = 0x75;

/// Display links check the id; game links permit their separate 0xFFFA RPC id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdPolicy {
    Display,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    ShortRecord,
    InvalidVersion(u8),
    InvalidLength { declared: usize, actual: usize },
    InvalidId(u16),
    PayloadTooLong,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid WireHeader2 message: {self:?}")
    }
}

impl std::error::Error for WireError {}

#[derive(Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub id: u16,
    pub payload: &'a [u8],
}

pub fn display_id_valid(id: u16) -> bool {
    (1..=0xFF).contains(&id) || id == 0xFFFE || id == 0xFFFF
}

fn check_id(id: u16, policy: IdPolicy) -> Result<(), WireError> {
    if policy == IdPolicy::Display && !display_id_valid(id) {
        return Err(WireError::InvalidId(id));
    }
    Ok(())
}

/// Parse a single decrypted DTLS application record without copying its payload.
/// The low version nibble and reserved bytes are ignored, matching upstream.
pub fn parse(record: &[u8], policy: IdPolicy) -> Result<Message<'_>, WireError> {
    if record.len() < HEADER_LEN {
        return Err(WireError::ShortRecord);
    }
    let version = record[0] >> 4;
    if version != 0 {
        return Err(WireError::InvalidVersion(version));
    }
    let id = u16::from_le_bytes([record[2], record[3]]);
    let total = u32::from_le_bytes([record[4], record[5], record[6], 0]) as usize;
    if total < HEADER_LEN || total != record.len() {
        return Err(WireError::InvalidLength {
            declared: total,
            actual: record.len(),
        });
    }
    check_id(id, policy)?;
    Ok(Message {
        id,
        payload: &record[HEADER_LEN..],
    })
}

pub fn pack(id: u16, payload: &[u8], policy: IdPolicy) -> Result<Vec<u8>, WireError> {
    check_id(id, policy)?;
    // Check before adding the header, including on 32-bit hosts.
    if payload.len() > MAX_TOTAL_LEN - HEADER_LEN {
        return Err(WireError::PayloadTooLong);
    }
    let total = (HEADER_LEN + payload.len()) as u32;
    let mut record = Vec::with_capacity(total as usize);
    record.extend_from_slice(&[0, 0]);
    record.extend_from_slice(&id.to_le_bytes());
    record.extend_from_slice(&total.to_le_bytes());
    record.extend_from_slice(payload);
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed bytes from the Python reference's documented little-endian format.
    #[test]
    fn display_rpc_golden_record() {
        let record = [0, 0, 0x75, 0, 11, 0, 0, 0, b'a', b'b', b'c'];
        assert_eq!(pack(ID_DISPLAY_RPC, b"abc", IdPolicy::Display).unwrap(), record);
        assert_eq!(parse(&record, IdPolicy::Display).unwrap(), Message {
            id: ID_DISPLAY_RPC,
            payload: b"abc",
        });
    }

    #[test]
    fn parser_ignores_only_the_documented_reserved_bits() {
        let mut record = [0x0F, 0xAA, 0x74, 0, 8, 0, 0, 0xFF];
        assert_eq!(parse(&record, IdPolicy::Display).unwrap().payload, b"");
        record[0] = 0x10;
        assert_eq!(parse(&record, IdPolicy::Display), Err(WireError::InvalidVersion(1)));
    }

    #[test]
    fn rejects_truncation_concatenation_and_invalid_lengths() {
        let record = [0, 0, 0x66, 0, 9, 0, 0, 0, 42];
        for len in 0..record.len() {
            assert!(parse(&record[..len], IdPolicy::Display).is_err());
        }
        let mut doubled = record.to_vec();
        doubled.extend_from_slice(&record);
        assert!(matches!(parse(&doubled, IdPolicy::Display), Err(WireError::InvalidLength { .. })));
        let too_small = [0, 0, 0x66, 0, 7, 0, 0, 0];
        assert!(matches!(parse(&too_small, IdPolicy::Display), Err(WireError::InvalidLength { .. })));
    }

    #[test]
    fn distinguishes_display_and_game_channels_for_every_id() {
        for id in 0..=u16::MAX {
            let valid = (1..=255).contains(&id) || id >= 0xFFFE;
            assert_eq!(pack(id, b"", IdPolicy::Display).is_ok(), valid);
            let game = pack(id, b"", IdPolicy::Game).unwrap();
            assert_eq!(parse(&game, IdPolicy::Display).is_ok(), valid);
            assert_eq!(parse(&game, IdPolicy::Game).unwrap().id, id);
        }
    }

    #[test]
    fn length_uses_all_three_bytes_and_payload_is_borrowed() {
        let payload = vec![0xAB; 65_536];
        let record = pack(ID_INPUT, &payload, IdPolicy::Game).unwrap();
        assert_eq!(&record[4..8], &[8, 0, 1, 0]);
        let message = parse(&record, IdPolicy::Game).unwrap();
        assert_eq!(message.payload, payload);
        assert_eq!(message.payload.as_ptr(), record[HEADER_LEN..].as_ptr());
    }

    #[test]
    fn enforces_the_24_bit_boundary() {
        let mut payload = vec![0; MAX_TOTAL_LEN - HEADER_LEN];
        let record = pack(ID_INPUT, &payload, IdPolicy::Game).unwrap();
        assert_eq!(&record[4..8], &[255, 255, 255, 0]);
        assert_eq!(parse(&record, IdPolicy::Game).unwrap().payload.len(), payload.len());
        payload.push(0);
        assert_eq!(pack(ID_INPUT, &payload, IdPolicy::Game), Err(WireError::PayloadTooLong));
    }
}
