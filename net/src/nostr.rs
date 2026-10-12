//! Nostr events as the rendezvous's envelope (sessions spec §3.1).
//!
//! The Nostr key is a throwaway per tab and authenticates nothing about the
//! game: every game message inside an event carries its own Ed25519
//! signature (§3's trust rule). Incoming Nostr signatures are therefore not
//! checked, and a frame carried in another seat's event (§4.3.2 rule 6) is
//! as good as the original.

use crate::bytes::{hex, sha256, Drbg};
use k256::schnorr::SigningKey;
use serde::{Deserialize, Serialize};

/// One kind for every Hyades event: regular (stored), and not in the NIP kind
/// list at the time of writing. *Placeholder.*
pub const EVENT_KIND: u32 = 7860;
/// Events expire after this long (NIP-40), so a relay may drop a finished
/// match. *Placeholder.*
pub const EXPIRY_SECONDS: u64 = 6 * 3600;
/// The topic prefix, versioned with the session protocol.
pub const TOPIC: &str = "hyades1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Event {
    /// The `t` tag values, which name the topics the event was published to.
    pub fn topics(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().filter(|t| t.len() >= 2 && t[0] == "t").map(|t| t[1].as_str())
    }
}

pub struct NostrKey {
    sk: SigningKey,
    pub pk: String,
}

impl NostrKey {
    pub fn generate(rng: &mut Drbg) -> NostrKey {
        loop {
            let b: [u8; 32] = rng.bytes();
            if let Ok(sk) = SigningKey::from_bytes(&b) {
                let pk = hex(&sk.verifying_key().to_bytes());
                return NostrKey { sk, pk };
            }
        }
    }
}

pub fn topic(name: &str) -> String {
    format!("{TOPIC}-{name}")
}

fn event_id(pubkey: &str, created_at: u64, kind: u32, tags: &[Vec<String>], content: &str) -> [u8; 32] {
    let ser = serde_json::to_string(&(0, pubkey, created_at, kind, tags, content)).expect("serializable");
    sha256(&[ser.as_bytes()])
}

/// Leading zero bits of an id (NIP-13).
pub fn leading_zero_bits(id: &[u8]) -> u32 {
    let mut n = 0;
    for &b in id {
        if b == 0 {
            n += 8;
        } else {
            return n + b.leading_zeros();
        }
    }
    n
}

/// Builds and signs an event on `topics`. With `pow > 0`, mines a NIP-13
/// nonce tag until the id has that many leading zero bits — the price of a
/// join request (§4.5 rule 5).
pub fn make_event(
    key: &NostrKey,
    topics: &[String],
    content: String,
    pow: u32,
    created_at: u64,
    rng: &mut Drbg,
) -> Event {
    let mut tags: Vec<Vec<String>> = topics.iter().map(|t| vec!["t".into(), t.clone()]).collect();
    tags.push(vec!["expiration".into(), (created_at + EXPIRY_SECONDS).to_string()]);
    let mut id = event_id(&key.pk, created_at, EVENT_KIND, &tags, &content);
    if pow > 0 {
        let base = tags.clone();
        let mut nonce: u64 = 0;
        loop {
            tags = base.clone();
            tags.push(vec!["nonce".into(), nonce.to_string(), pow.to_string()]);
            id = event_id(&key.pk, created_at, EVENT_KIND, &tags, &content);
            if leading_zero_bits(&id) >= pow {
                break;
            }
            nonce += 1;
        }
    }
    let aux: [u8; 32] = rng.bytes();
    let sig = key.sk.sign_raw(&id, &aux).expect("a 32-byte message signs");
    Event {
        id: hex(&id),
        pubkey: key.pk.clone(),
        created_at,
        kind: EVENT_KIND,
        tags,
        content,
        sig: hex(&sig.to_bytes()),
    }
}

/// Whether an event's id is the hash of its contents and meets its declared
/// proof-of-work. The host checks a join request this way before acting on it.
pub fn id_and_pow_ok(e: &Event, min_pow: u32) -> bool {
    let id = event_id(&e.pubkey, e.created_at, e.kind, &e.tags, &e.content);
    hex(&id) == e.id && leading_zero_bits(&id) >= min_pow
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytes::from_hex;
    use k256::schnorr::{Signature, VerifyingKey};

    #[test]
    fn bip340_test_vector_0() {
        let sk = SigningKey::from_bytes(&from_hex(&format!("{:064x}", 3)).unwrap()).unwrap();
        assert_eq!(
            hex(&sk.verifying_key().to_bytes()),
            "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9"
        );
        let sig = sk.sign_raw(&[0u8; 32], &[0u8; 32]).unwrap();
        assert_eq!(
            hex(&sig.to_bytes()).to_uppercase(),
            "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA821525F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0"
        );
    }

    #[test]
    fn an_event_signs_verifies_and_meets_its_pow() {
        let mut rng = Drbg::new([4; 32]);
        let key = NostrKey::generate(&mut rng);
        let e = make_event(&key, &[topic("room-x")], "{}".into(), 8, 1_800_000_000, &mut rng);
        assert!(id_and_pow_ok(&e, 8));
        assert_eq!(e.topics().collect::<Vec<_>>(), vec!["hyades1-room-x"]);
        let vk = VerifyingKey::from_bytes(&from_hex(&e.pubkey).unwrap()).unwrap();
        let sig = Signature::try_from(from_hex(&e.sig).unwrap().as_slice()).unwrap();
        vk.verify_raw(&from_hex(&e.id).unwrap(), &sig).unwrap();
        let mut t = e.clone();
        t.content = "{\"x\":1}".into();
        assert!(!id_and_pow_ok(&t, 0), "the id covers the content");
    }

    #[test]
    fn zero_bits() {
        assert_eq!(leading_zero_bits(&[0, 0, 0x10]), 19);
        assert_eq!(leading_zero_bits(&[0x80]), 0);
        assert_eq!(leading_zero_bits(&[0, 1]), 15);
    }
}
