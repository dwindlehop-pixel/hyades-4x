//! The native half of the parity check (T-128): prints one line per case,
//! the same lines `compare.mjs` reads back from the wasm32 module.
use hyades_parity::{case_line, CASES};
use std::io::Write;

fn main() {
    for case in CASES {
        println!("{}", case_line(case));
        let _ = std::io::stdout().flush();
    }
}
