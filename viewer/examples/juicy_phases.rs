use hyades_viewer::juicy::*;
use hyades_viewer::raster::Raster;
use std::time::Instant;
fn main() {
    let (w, h) = (960, 600);
    let mut hdr = Hdr::new(w, h);
    let lut = ToneLut::new();
    let mut out = Raster::new(w, h);
    let n = 50;
    let t = Instant::now();
    for _ in 0..n {
        hdr.px.iter_mut().for_each(|p| *p = [0.0; 3]);
    }
    println!("clear      {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / n as f64);
    let t = Instant::now();
    let mut b = Hdr::new(1, 1);
    for _ in 0..n {
        b = hdr.downsample(4);
    }
    println!("downsample {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / n as f64);
    let t = Instant::now();
    for _ in 0..n {
        b.blur(2, 3);
    }
    println!("blur       {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / n as f64);
    let t = Instant::now();
    for _ in 0..n {
        hdr.add_upsampled(&b, 4, 1.0);
    }
    println!("upsample   {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / n as f64);
    let t = Instant::now();
    for _ in 0..n {
        hdr.resolve(&mut out, &lut);
    }
    println!("resolve    {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / n as f64);
    let t = Instant::now();
    for i in 0..n * 300 {
        hdr.splat((i % 900) as f64, (i % 500) as f64, 1.5, [1.0; 3], 1.0);
    }
    println!("splat r1.5 {:.4} ms each", t.elapsed().as_secs_f64() * 1e3 / (n * 300) as f64);
}
