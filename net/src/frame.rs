//! The 144-byte peer frame (docs/Hyades_netcode.md §4).
//!
//! ```text
//! offset size field
//!  0      1   version — FRAME_VERSION
//!  1      1   kind    — Kind
//!  2      2   seat    (u16 LE)
//!  4      4   round   (u32 LE)
//!  8     32   session_id
//! 40     32   payload, kind-specific, zero-padded
//! 72      8   reserved, zero
//! 80     64   Ed25519 signature over bytes 0..80
//! ```
//!
//! Signing happens outside this crate, in WebCrypto, so a seat's private key
//! never enters wasm memory (sessions spec §4.1): this module builds the 80
//! signed bytes and assembles the frame once the signature comes back.
//! Verification happens here, with one implementation on every client, so
//! that no two browsers can disagree about which frames are valid.

use crate::bytes::sha256;
use ed25519_dalek::{Signature, VerifyingKey};

pub const FRAME_LEN: usize = 144;
pub const BODY_LEN: usize = 80;
pub const FRAME_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Commit = 1,
    Reveal = 2,
    Checkpoint = 3,
    TimeoutVote = 5,
}

impl Kind {
    fn from_u8(b: u8) -> Option<Kind> {
        Some(match b {
            1 => Kind::Commit,
            2 => Kind::Reveal,
            3 => Kind::Checkpoint,
            5 => Kind::TimeoutVote,
            _ => return None,
        })
    }

    /// Payload bytes the kind may use; the rest must be zero (netcode §4.3 step 4).
    fn used(self) -> usize {
        match self {
            Kind::Commit | Kind::Checkpoint => 32,
            Kind::Reveal => 23,
            Kind::TimeoutVote => 3,
        }
    }
}

/// The phase a timeout vote names (netcode §5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    Commit = 0,
    Reveal = 1,
}

/// An order: `card` indexes the card list, `PASS` is the default order every
/// client substitutes for a missing or illegal one (netcode §5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub card: u16,
    pub target_kind: u8,
    pub target_ref: u32,
}

/// The number of cards in the tier-0 list (`src/cards.rs`); any other id but
/// `PASS` coerces to `PASS`.
pub const CARD_COUNT: u16 = 18;

pub const PASS: Order = Order { card: 0xffff, target_kind: 0, target_ref: 0 };

impl Order {
    pub fn bytes(&self) -> [u8; 7] {
        let mut b = [0u8; 7];
        b[0..2].copy_from_slice(&self.card.to_le_bytes());
        b[2] = self.target_kind;
        b[3..7].copy_from_slice(&self.target_ref.to_le_bytes());
        b
    }

    /// Coerce to legality, never reject (netcode §5.1). Without the engine
    /// in the browser the only legality checked is that the card exists.
    pub fn coerce(self) -> Order {
        if self.card < CARD_COUNT {
            self
        } else {
            PASS
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub kind: Kind,
    pub seat: u16,
    pub round: u32,
    pub payload: [u8; 32],
    pub bytes: [u8; FRAME_LEN],
}

impl Frame {
    pub fn key(&self) -> FrameKey {
        FrameKey { seat: self.seat, round: self.round, kind: self.kind, extra: self.extra() }
    }

    /// A timeout vote is one frame per (voter, round, subject, phase): the
    /// netcode key (seat, round, kind) would let a seat vote once a round.
    fn extra(&self) -> u32 {
        match self.kind {
            Kind::TimeoutVote => {
                let v = read_vote(&self.payload);
                ((v.0 as u32) << 8) | v.1 as u32
            }
            _ => 0,
        }
    }
}

/// What makes two frames the same frame: equivocation is two differing
/// frames under one key (netcode §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameKey {
    pub seat: u16,
    pub round: u32,
    pub kind: Kind,
    pub extra: u32,
}

/// The 80 bytes a frame's signature covers.
pub fn body(kind: Kind, seat: u16, round: u32, session: &[u8; 32], payload: &[u8]) -> [u8; BODY_LEN] {
    assert!(payload.len() <= kind.used(), "payload too long for {kind:?}");
    let mut b = [0u8; BODY_LEN];
    b[0] = FRAME_VERSION;
    b[1] = kind as u8;
    b[2..4].copy_from_slice(&seat.to_le_bytes());
    b[4..8].copy_from_slice(&round.to_le_bytes());
    b[8..40].copy_from_slice(session);
    b[40..40 + payload.len()].copy_from_slice(payload);
    b
}

pub fn assemble(body: &[u8; BODY_LEN], sig: &[u8; 64]) -> [u8; FRAME_LEN] {
    let mut f = [0u8; FRAME_LEN];
    f[..BODY_LEN].copy_from_slice(body);
    f[BODY_LEN..].copy_from_slice(sig);
    f
}

/// Netcode §4.3's receive discipline, cheapest check first: length, version,
/// session, kind, zero padding, seat range, then the signature. Returns
/// `None` at the first failure.
pub fn decode(bytes: &[u8], session: &[u8; 32], seats: &[VerifyingKey]) -> Option<Frame> {
    let bytes: [u8; FRAME_LEN] = bytes.try_into().ok()?;
    if bytes[0] != FRAME_VERSION || &bytes[8..40] != session {
        return None;
    }
    let kind = Kind::from_u8(bytes[1])?;
    if bytes[40 + kind.used()..80].iter().any(|&b| b != 0) {
        return None;
    }
    let seat = u16::from_le_bytes([bytes[2], bytes[3]]);
    let key = seats.get(seat as usize)?;
    let sig = Signature::from_bytes(bytes[80..].try_into().ok()?);
    key.verify_strict(&bytes[..80], &sig).ok()?;
    Some(Frame {
        kind,
        seat,
        round: u32::from_le_bytes(bytes[4..8].try_into().ok()?),
        payload: bytes[40..72].try_into().ok()?,
        bytes,
    })
}

/// ORDER_COMMIT's payload: SHA-256(session ‖ round ‖ seat ‖ order ‖ salt) (netcode §4.2).
pub fn commit_payload(session: &[u8; 32], round: u32, seat: u16, order: &Order, salt: &[u8; 16]) -> [u8; 32] {
    sha256(&[session, &round.to_le_bytes(), &seat.to_le_bytes(), &order.bytes(), salt])
}

/// ORDER_REVEAL's payload: the order and its 128-bit salt.
pub fn reveal_payload(order: &Order, salt: &[u8; 16]) -> [u8; 23] {
    let mut p = [0u8; 23];
    p[..7].copy_from_slice(&order.bytes());
    p[7..].copy_from_slice(salt);
    p
}

pub fn read_reveal(payload: &[u8; 32]) -> (Order, [u8; 16]) {
    let order = Order {
        card: u16::from_le_bytes([payload[0], payload[1]]),
        target_kind: payload[2],
        target_ref: u32::from_le_bytes([payload[3], payload[4], payload[5], payload[6]]),
    };
    (order, payload[7..23].try_into().expect("16 bytes"))
}

pub fn vote_payload(subject: u16, phase: Phase) -> [u8; 3] {
    let s = subject.to_le_bytes();
    [s[0], s[1], phase as u8]
}

pub fn read_vote(payload: &[u8; 32]) -> (u16, Phase) {
    let phase = if payload[2] == 0 { Phase::Commit } else { Phase::Reveal };
    (u16::from_le_bytes([payload[0], payload[1]]), phase)
}

/// The checkpoint root while the engine is not in the browser (T-165): a
/// hash of the order every seat applied in a round, in seat order — the
/// "inputs leaf" of sessions spec R-SES11. Two clients that resolved a round
/// differently sign different roots.
pub fn inputs_root(session: &[u8; 32], round: u32, applied: &[Order]) -> [u8; 32] {
    let orders: Vec<u8> = applied.iter().flat_map(|o| o.bytes()).collect();
    sha256(&[session, &round.to_le_bytes(), &orders])
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn frame(sk: &SigningKey, kind: Kind, seat: u16, session: &[u8; 32], payload: &[u8]) -> [u8; FRAME_LEN] {
        let b = body(kind, seat, 3, session, payload);
        assemble(&b, &sk.sign(&b).to_bytes())
    }

    #[test]
    fn a_signed_frame_decodes_and_a_tampered_one_does_not() {
        let keys: Vec<SigningKey> = (0..3u8).map(|i| SigningKey::from_bytes(&[i + 1; 32])).collect();
        let pubs: Vec<VerifyingKey> = keys.iter().map(|k| k.verifying_key()).collect();
        let session = [9u8; 32];
        let order = Order { card: 4, target_kind: 1, target_ref: 77 };
        let salt = [5u8; 16];
        let f = frame(&keys[1], Kind::Reveal, 1, &session, &reveal_payload(&order, &salt));
        let d = decode(&f, &session, &pubs).expect("valid frame");
        assert_eq!((d.kind, d.seat, d.round), (Kind::Reveal, 1, 3));
        assert_eq!(read_reveal(&d.payload), (order, salt));

        assert!(decode(&f[..143], &session, &pubs).is_none(), "length");
        assert!(decode(&f, &[0; 32], &pubs).is_none(), "session");
        for at in [0usize, 1, 2, 5, 40, 63, 79, 100] {
            let mut t = f;
            t[at] ^= 1;
            assert!(decode(&t, &session, &pubs).is_none(), "byte {at} flipped");
        }
        // Signed by seat 2's key while claiming seat 1.
        let forged = frame(&keys[2], Kind::Reveal, 1, &session, &reveal_payload(&order, &salt));
        assert!(decode(&forged, &session, &pubs).is_none(), "wrong signer");
        // A seat outside the table.
        let out = frame(&keys[0], Kind::Reveal, 7, &session, &[]);
        assert!(decode(&out, &session, &pubs).is_none(), "seat range");
    }

    #[test]
    fn non_zero_padding_is_a_violation() {
        let k = SigningKey::from_bytes(&[1; 32]);
        let session = [1u8; 32];
        let mut b = body(Kind::TimeoutVote, 0, 0, &session, &vote_payload(2, Phase::Reveal));
        b[50] = 1; // inside the vote's unused payload
        let f = assemble(&b, &k.sign(&b).to_bytes());
        assert!(decode(&f, &session, &[k.verifying_key()]).is_none());
    }

    #[test]
    fn commit_binds_order_and_salt() {
        let s = [3u8; 32];
        let a = commit_payload(&s, 1, 0, &Order { card: 1, target_kind: 0, target_ref: 0 }, &[0; 16]);
        assert_ne!(a, commit_payload(&s, 1, 0, &Order { card: 2, target_kind: 0, target_ref: 0 }, &[0; 16]));
        assert_ne!(a, commit_payload(&s, 1, 0, &Order { card: 1, target_kind: 0, target_ref: 0 }, &[1; 16]));
        assert_ne!(a, commit_payload(&s, 2, 0, &Order { card: 1, target_kind: 0, target_ref: 0 }, &[0; 16]));
    }

    #[test]
    fn unknown_cards_coerce_to_pass() {
        assert_eq!(Order { card: 18, target_kind: 0, target_ref: 0 }.coerce(), PASS);
        let ok = Order { card: 17, target_kind: 0, target_ref: 0 };
        assert_eq!(ok.coerce(), ok);
    }
}
