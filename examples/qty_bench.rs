//! **Does wrapping an `f64` cost anything? — the five-second version.**
//!
//! `units::Qty<S>` is `#[repr(transparent)]` over one `f64` with a zero-sized
//! ladder marker, so arithmetic should compile to exactly what the bare `f64`
//! compiles to. The unit test `arithmetic_costs_what_an_f64_costs` guards that
//! in a fraction of a second; this runs each loop for ~5 s so the number is
//! stable enough to quote.
//!
//! It also times the thing that is **not** free, and that separation is the
//! whole point: storage is kilotons and a Band is a *reading*, so
//!
//! - `+ - * / min max` touch only the stored `f64` — parity with bare `f64`;
//! - `band()` is a `ln` and `at_band()` a `powf` — one to two orders of
//!   magnitude more expensive.
//!
//! Which is why the engine stores the mass and converts at the edges. It is
//! also why `Factors::bio_max_band` exists as a cache today (R-O70): a
//! conversion had got onto the hot path. Denominating `K` in mass removes it.
//!
//! Run: `cargo run --release --example qty_bench`
use hyades_engine::units::{Band, BandTier, Cost, Kilotons, Qty};
use std::hint::black_box;
use std::io::Write;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_secs(5);
const CHUNK: usize = 1 << 16;

/// Run `body` in chunks until the budget is spent; report nanoseconds per op.
fn per_op(label: &str, mut body: impl FnMut(usize)) -> f64 {
    let start = Instant::now();
    let mut ops = 0usize;
    while start.elapsed() < BUDGET {
        for i in 0..CHUNK {
            body(i);
        }
        ops += CHUNK;
    }
    let ns = start.elapsed().as_secs_f64() * 1e9 / ops as f64;
    println!("  {label:<34} {ns:>8.3} ns/op   ({ops} ops)");
    std::io::stdout().flush().ok();
    ns
}

fn main() {
    println!("qty_bench — {BUDGET:?} per loop, release build recommended\n");

    let mut a = 1.0f64;
    let raw = per_op("bare f64: mul/add/min/max", |i| {
        let step = 0.5 + (i % 17) as f64;
        a = ((a + step) * 0.999 - step * 0.5).clamp(-1e12, 1e12);
        black_box(a);
    });

    let mut b = Kilotons::new(1.0);
    let wrapped = per_op("Qty<Mass>: the same, wrapped", |i| {
        let step = Kilotons::new(0.5 + (i % 17) as f64);
        b = ((b + step) * 0.999 - step * 0.5).clamp(Kilotons::new(-1e12), Kilotons::new(1e12));
        black_box(b);
    });

    let mut c = Qty::<Cost>::new(1.0);
    let cost = per_op("Qty<Cost>: the other ladder", |i| {
        let step = Qty::<Cost>::new(0.5 + (i % 17) as f64);
        c = ((c + step) * 0.999 - step * 0.5).clamp(Qty::<Cost>::new(-1e12), Qty::<Cost>::new(1e12));
        black_box(c);
    });

    println!();
    let read = per_op("Qty::band() — a ln", |i| {
        black_box(Kilotons::new(1.0 + (i % 4096) as f64).band());
    });
    let write = per_op("Qty::at_band() — a powf", |i| {
        black_box(Kilotons::at_band(Band::new((i % 4096) as f64 / 1024.0)));
    });
    let tier = per_op("Qty::at(tier, fraction)", |i| {
        black_box(Kilotons::at(BandTier::II, (i % 1024) as f64 / 1024.0));
    });

    println!("\n  arithmetic, Qty<Mass> / f64   {:.3}x", wrapped / raw);
    println!("  arithmetic, Qty<Cost> / f64   {:.3}x", cost / raw);
    println!("  a Band read  costs            {:.1}x an arithmetic op", read / wrapped);
    println!("  a Band write costs            {:.1}x an arithmetic op", write / wrapped);
    println!("  at(tier, fraction) costs      {:.1}x an arithmetic op", tier / wrapped);
    println!(
        "\nStorage is the mass because the first two rows are free and the last three\n\
         are not. Convert at the edges; never inside a loop over entities."
    );
}
