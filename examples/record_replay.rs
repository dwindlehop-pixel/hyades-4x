//! **Write the demo games' replays to files** (`docs/Hyades_interface.md`
//! §2, §8).
//!
//! The client records a demo in the browser from its recipe
//! (`hyades_engine::demos`, T-180); this writes the same bytes to
//! `<out>/<name>.json`, with an `<out>/index.json` listing them, for the
//! format's tests and for opening a replay as a file.
//!
//! Run: `cargo run --release --example record_replay -- <out dir> [--quick]`.
//! `--quick` records short, small versions — the fixture the viewer's tests
//! read in CI.
use hyades_engine::demos::demos;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "web/replays".into());
    let quick = args.iter().any(|a| a == "--quick");
    std::fs::create_dir_all(&out).expect("create the output directory");
    let mut index = String::from("[");
    for (i, d) in demos(quick).iter().enumerate() {
        let started = std::time::Instant::now();
        let mut r = d.recorder();
        let frames = r.frames_total();
        r.step(usize::MAX);
        let json = r.finish();
        let file = format!("{}.json", d.name);
        std::fs::write(format!("{out}/{file}"), &json).expect("write the replay");
        println!(
            "{:<10} {:>7.1} kB  {:>5} frames  {:.1} s",
            d.name,
            json.len() as f64 / 1024.0,
            frames,
            started.elapsed().as_secs_f64()
        );
        std::io::stdout().flush().ok();
        if i > 0 {
            index.push(',');
        }
        index.push_str(&format!(
            "{{\"file\":\"{file}\",\"name\":\"{}\",\"label\":\"{}\",\"seats\":{},\"horizon_years\":{},\"bytes\":{}}}",
            d.name,
            d.label,
            d.seats,
            d.horizon,
            json.len()
        ));
    }
    index.push(']');
    std::fs::write(format!("{out}/index.json"), index).expect("write the index");
}
