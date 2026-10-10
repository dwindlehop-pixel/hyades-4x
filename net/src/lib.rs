//! `hyades-net`: sessions and transport for the Hyades client
//! (docs/Hyades_sessions_discovery_and_security.md).
//!
//! The protocol is a state machine that does no I/O. The host — the browser
//! through `web`, or a simulated network in the tests — feeds it relay
//! messages, signatures and the time, and carries out what it asks for:
//! open a relay, send text to a relay, sign bytes. Every rule that decides
//! what to send, when, and what a round resolved to lives here, where
//! `cargo test` can drive it with simulated relays and a simulated clock.
//!
//! It does not link the engine (design law #15): until the engine runs in
//! the browser (T-165) a round's checkpoint is the hash of the orders every
//! seat applied, not a state root.

pub mod bytes;
pub mod frame;
pub mod link;
pub mod nostr;
pub mod relay;
pub mod session;

#[cfg(target_arch = "wasm32")]
pub mod web;
