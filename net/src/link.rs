//! The link payload (sessions spec §5.1): everything after `#j=` in
//! `https://<pages host>/<repo>/#j=<payload>`. It carries no network address
//! (ruling 7); the room is found on the rendezvous by its id.
//!
//! ```text
//! offset size field
//!  0      1   format version — LINK_VERSION
//!  1     32   engine_sha256 — all zero: the transport test, no engine (T-165)
//! 33     16   room_id
//! 49     32   host_pubkey — the originator's Ed25519 key
//! 81      8   params: seats u8, round_s u16, rounds u8, patience_s u16, flags u8, reserved u8
//! ```

use crate::bytes::{b64url, from_b64url};
use serde::Serialize;

pub const LINK_VERSION: u8 = 1;
pub const LINK_LEN: usize = 89;
pub const MIN_SEATS: u8 = 2;
pub const MAX_SEATS: u8 = 18;

/// The parameters the originator sets (ruling 1). Magnitudes are
/// *placeholders*; the defaults are §4.3.1's 180 s floor and §4.3.2's 120 s
/// patience over netcode's ten barriers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Params {
    pub seats: u8,
    /// The floor on wall time per round, seconds (§4.3.1 rule 2).
    pub round_s: u16,
    pub rounds: u8,
    /// How long after a phase opens a seat may vote a timeout, seconds (§4.3.2 rule 7).
    pub patience_s: u16,
    /// Listed in Open games (§5.3).
    pub public: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params { seats: 6, round_s: 180, rounds: 10, patience_s: 120, public: false }
    }
}

impl Params {
    pub fn clamped(self) -> Params {
        Params {
            seats: self.seats.clamp(MIN_SEATS, MAX_SEATS),
            round_s: self.round_s.clamp(1, 3600),
            rounds: self.rounds.clamp(1, 60),
            patience_s: self.patience_s.clamp(1, 3600),
            public: self.public,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkData {
    pub room: [u8; 16],
    pub host: [u8; 32],
    pub params: Params,
}

impl LinkData {
    pub fn encode(&self) -> String {
        let mut b = [0u8; LINK_LEN];
        b[0] = LINK_VERSION;
        b[33..49].copy_from_slice(&self.room);
        b[49..81].copy_from_slice(&self.host);
        let p = &self.params;
        b[81] = p.seats;
        b[82..84].copy_from_slice(&p.round_s.to_le_bytes());
        b[84] = p.rounds;
        b[85..87].copy_from_slice(&p.patience_s.to_le_bytes());
        b[87] = p.public as u8;
        b64url(&b)
    }

    /// Reads a payload, or a whole link containing `#j=<payload>`.
    pub fn decode(link: &str) -> Result<LinkData, String> {
        let payload = match link.find("#j=") {
            Some(i) => &link[i + 3..],
            None => link.trim(),
        };
        let b = from_b64url(payload).ok_or("the link is not base64url")?;
        if b.len() != LINK_LEN {
            return Err(format!("the link holds {} bytes, not {LINK_LEN}", b.len()));
        }
        if b[0] != LINK_VERSION {
            return Err(format!("link format {} is not {LINK_VERSION}", b[0]));
        }
        if b[1..33].iter().any(|&x| x != 0) {
            return Err("the link names an engine build; this client runs the transport test only".into());
        }
        let params = Params {
            seats: b[81],
            round_s: u16::from_le_bytes([b[82], b[83]]),
            rounds: b[84],
            patience_s: u16::from_le_bytes([b[85], b[86]]),
            public: b[87] & 1 == 1,
        };
        if params.clamped() != params {
            return Err("the link's parameters are out of range".into());
        }
        Ok(LinkData { room: b[33..49].try_into().unwrap(), host: b[49..81].try_into().unwrap(), params })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_fits_a_twitch_message() {
        let d = LinkData {
            room: [7; 16],
            host: [9; 32],
            params: Params { seats: 18, round_s: 180, rounds: 10, patience_s: 120, public: true },
        };
        let payload = d.encode();
        assert_eq!(LinkData::decode(&payload).unwrap(), d);
        let url = format!("https://dwindlehop-pixel.github.io/hyades-4x/#j={payload}");
        assert_eq!(LinkData::decode(&url).unwrap(), d);
        assert_eq!(url.len(), 167, "sessions spec §5.1's computed length");
        assert!(url.len() <= 500);
    }

    #[test]
    fn rejects_what_it_cannot_read() {
        assert!(LinkData::decode("!!").is_err());
        assert!(LinkData::decode(&b64url(&[1u8; 10])).is_err());
        let mut b = [0u8; LINK_LEN];
        b[0] = 1;
        b[81] = 40; // 40 seats
        assert!(LinkData::decode(&b64url(&b)).is_err());
    }
}
