//! **The Hyades replay viewer** (`docs/Hyades_interface.md`).
//!
//! Reads a replay the engine recorded (`hyades_engine::replay`), plays it
//! forward and back, filters its log, and draws it into a pixel framebuffer in
//! one of two modes — **tactical**, every entity at its position as a glyph of
//! its Design in its seat's and Doctrine's colors, and **juicy**, the same
//! state as light. It does not link the engine: what it knows of a game is
//! the replay, so nothing here can reach the simulation (design law #15).
//!
//! Compiled to `wasm32-unknown-unknown` it is the module the web shell
//! (`web/`) loads, through [`ffi`]'s plain C interface.

pub mod app;
pub mod camera;
pub mod color;
pub mod ffi;
pub mod glyph;
pub mod json;
pub mod juicy;
pub mod logview;
pub mod palette;
pub mod raster;
pub mod replay;
pub mod tactical;
pub mod timeline;
