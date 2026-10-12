//! Byte helpers: base64url, hex, SHA-256, canonical JSON, and the
//! deterministic random generator the state machine draws salts and jitter
//! from once the host has seeded it with real entropy.

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use sha2::{Digest, Sha256};

pub fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn from_b64url(s: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(s.trim_end_matches('=')).ok()
}

pub fn b64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn from_b64(s: &str) -> Option<Vec<u8>> {
    STANDARD.decode(s).ok()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len() / 2).map(|i| u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()).collect()
}

/// `from_hex` into a fixed-size array.
pub fn hex_array<const N: usize>(s: &str) -> Option<[u8; N]> {
    from_hex(s)?.try_into().ok()
}

pub fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// JSON with object keys sorted at every level: the bytes a lobby signature
/// covers. `serde_json::Value` keeps object keys in a sorted map unless the
/// `preserve_order` feature is on, and this crate does not turn it on, so
/// serializing a `Value` is canonical.
pub fn canonical_json(v: &serde_json::Value) -> String {
    v.to_string()
}

/// A hash-based generator: block `n` is SHA-256(seed ‖ n). Seeded by the
/// host from the platform's cryptographic source (WebCrypto in the browser),
/// so salts drawn from it are as unpredictable as the seed; seeded with a
/// constant in tests, so a simulated match is reproducible.
#[derive(Clone)]
pub struct Drbg {
    seed: [u8; 32],
    counter: u64,
}

impl Drbg {
    pub fn new(seed: [u8; 32]) -> Self {
        Drbg { seed, counter: 0 }
    }

    pub fn fill(&mut self, out: &mut [u8]) {
        for chunk in out.chunks_mut(32) {
            let block = sha256(&[&self.seed, &self.counter.to_le_bytes()]);
            self.counter += 1;
            chunk.copy_from_slice(&block[..chunk.len()]);
        }
    }

    pub fn bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut b = [0u8; N];
        self.fill(&mut b);
        b
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        let b: [u8; 8] = self.bytes();
        (u64::from_le_bytes(b) >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let b = [0u8, 1, 2, 250, 255];
        assert_eq!(from_hex(&hex(&b)).unwrap(), b);
        assert_eq!(from_b64url(&b64url(&b)).unwrap(), b);
        assert_eq!(from_b64(&b64(&b)).unwrap(), b);
        assert!(from_hex("abc").is_none());
        assert!(from_hex("zz").is_none());
    }

    #[test]
    fn canonical_json_sorts_keys() {
        let v: serde_json::Value = serde_json::from_str(r#"{"b":1,"a":{"d":2,"c":3}}"#).unwrap();
        assert_eq!(canonical_json(&v), r#"{"a":{"c":3,"d":2},"b":1}"#);
    }

    #[test]
    fn drbg_is_reproducible_and_spread() {
        let mut a = Drbg::new([7; 32]);
        let mut b = Drbg::new([7; 32]);
        assert_eq!(a.bytes::<48>(), b.bytes::<48>());
        let mean = (0..1000).map(|_| a.unit()).sum::<f64>() / 1000.0;
        assert!((0.45..0.55).contains(&mean), "{mean}");
    }
}
