#[macro_use]
extern crate criterion;

use criterion::Criterion;
use nessim::SimulationState;
use std::fs::File;

fn criterion_benchmark(c: &mut Criterion) {
    let mut sim = SimulationState::new();
    sim.load_rom(&mut File::open("test_data/scanline.nes").unwrap());
    c.bench_function("100 Half-Steps", move |b| {
        b.iter(|| {
            for _ in 0..100 {
                sim.half_step()
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
