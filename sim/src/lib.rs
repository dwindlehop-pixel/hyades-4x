//! **The engine as a browser module** (`docs/Hyades_interface.md` §2, T-180;
//! the first half of T-165).
//!
//! The engine is bit-deterministic, native and wasm32 alike (T-127), so a demo
//! game ships as its recipe (`hyades_engine::demos`) and is recorded where it
//! is watched: a web worker loads this module, records a few frames at a time
//! so the page can show progress, and hands the finished replay to the viewer.
//! The viewer still links nothing but the replay format (design law #15).
//!
//! A plain C interface, as the viewer's (`viewer/src/ffi.rs`): every function
//! takes and returns numbers, and bytes cross through buffers the module owns.
//!
//! - **In:** [`hs_input`] returns a buffer of the asked length; the worker
//!   copies a demo's name into it and calls [`hs_begin`].
//! - **Out:** [`hs_demos`] and [`hs_finish`] return UTF-8 text of
//!   [`hs_text_len`] bytes.
//!
//! A pointer is valid until the next call that writes the same buffer.

use hyades_engine::demos::{demo, demos};
use hyades_engine::replay::Recorder;
use std::cell::RefCell;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static TEXT: RefCell<String> = const { RefCell::new(String::new()) };
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

fn text(s: String) -> *const u8 {
    TEXT.with(|t| {
        *t.borrow_mut() = s;
        t.borrow().as_ptr()
    })
}

/// A buffer of `len` bytes for the worker to write into.
#[no_mangle]
pub extern "C" fn hs_input(len: usize) -> *mut u8 {
    INPUT.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        b.resize(len, 0);
        b.as_mut_ptr()
    })
}

/// The bytes of the text last returned.
#[no_mangle]
pub extern "C" fn hs_text_len() -> usize {
    TEXT.with(|t| t.borrow().len())
}

/// **The demos**, one a line: `name \t label \t seats \t horizon years`.
/// `quick` (nonzero) lists the short, small versions the tests record.
#[no_mangle]
pub extern "C" fn hs_demos(quick: i32) -> *const u8 {
    text(demos(quick != 0).iter().map(|d| format!("{}\t{}\t{}\t{}\n", d.name, d.label, d.seats, d.horizon)).collect())
}

/// **Starts recording** the demo named in the input buffer. Returns the
/// frames the replay will hold, or 0 for a name no demo has.
#[no_mangle]
pub extern "C" fn hs_begin(quick: i32) -> u32 {
    let name = INPUT.with(|b| String::from_utf8_lossy(&b.borrow()).into_owned());
    let recorder = demo(&name, quick != 0).map(|d| d.recorder());
    let frames = recorder.as_ref().map_or(0, |r| r.frames_total() as u32);
    RECORDER.with(|r| *r.borrow_mut() = recorder);
    frames
}

/// **Records up to `frames` more frames**; returns the frames recorded so far.
#[no_mangle]
pub extern "C" fn hs_step(frames: u32) -> u32 {
    RECORDER.with(|r| r.borrow_mut().as_mut().map_or(0, |r| r.step(frames as usize) as u32))
}

/// **The replay**, as text — every frame recorded so far, then the hulls and
/// the events. Ends the recording.
#[no_mangle]
pub extern "C" fn hs_finish() -> *const u8 {
    text(RECORDER.with(|r| r.borrow_mut().take().map(Recorder::finish).unwrap_or_default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn begin(name: &str) -> u32 {
        let p = hs_input(name.len());
        // SAFETY: `hs_input` returned a buffer of exactly `name.len()` bytes.
        unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), p, name.len()) };
        hs_begin(1)
    }

    fn finish() -> String {
        let p = hs_finish();
        // SAFETY: `hs_finish` returned `hs_text_len()` bytes of UTF-8.
        let bytes = unsafe { std::slice::from_raw_parts(p, hs_text_len()) };
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// **A demo recorded a few frames at a time through the module is the
    /// replay recorded whole** (T-180): the recipe is the replay.
    #[test]
    fn a_demo_recorded_in_steps_is_the_demo_recorded_whole() {
        for d in demos(true) {
            let total = begin(d.name);
            assert!(total > 1, "{} has frames", d.name);
            let mut done = 0;
            while done < total {
                let next = hs_step(7);
                assert!(next > done || next == total, "{} advances", d.name);
                done = next;
            }
            let whole = {
                let mut r = d.recorder();
                r.step(usize::MAX);
                r.finish()
            };
            assert_eq!(finish(), whole, "{}", d.name);
        }
    }

    #[test]
    fn the_demo_list_names_every_demo_and_an_unknown_name_records_nothing() {
        let p = hs_demos(1);
        // SAFETY: `hs_demos` returned `hs_text_len()` bytes of UTF-8.
        let list = String::from_utf8(unsafe { std::slice::from_raw_parts(p, hs_text_len()) }.to_vec()).unwrap();
        let names: Vec<&str> = list.lines().map(|l| l.split('\t').next().unwrap()).collect();
        assert_eq!(names, demos(true).iter().map(|d| d.name).collect::<Vec<_>>());
        assert_eq!(begin("no such demo"), 0);
        assert_eq!(hs_step(10), 0);
        assert_eq!(finish(), "");
    }
}
