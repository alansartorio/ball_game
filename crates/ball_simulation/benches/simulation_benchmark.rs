use ball_simulation::SimulationState;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let state = SimulationState {
        time: 0.0,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![],
        blocks: vec![],
    };
    c.bench_function("simulation iteration", |b| {
        b.iter(|| SimulationState::next(black_box(state.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
