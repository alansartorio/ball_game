use ball_simulation::{Ball, SimulationState};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Vector2;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut balls = vec![];

    for y in 1..=3 {
        for x in 1..=9 {
            balls.push(Ball {
                position: Vector2::new(0.1 * x as f64, 0.1 * y as f64),
                velocity: Vector2::new(0.1 * y as f64, 0.1 * x as f64),
                radius: 0.04,
            });
        }
    }

    let mut blocks = vec![];

    for y in 6..=10 {
        let y = y as f64 / 11.0;
        for x in 1..=10 {
            let x = x as f64 / 11.0;
            blocks.push(ball_simulation::Block {
                min_y: y - 0.04,
                max_y: y + 0.04,
                min_x: x - 0.04,
                max_x: x + 0.04,
            });
        }
    }

    let state = SimulationState {
        time: 0.0,
        space_width: 1.0,
        space_height: 1.0,
        balls,
        blocks,
    };
    c.bench_function("simulation iteration", |b| {
        b.iter(|| SimulationState::next(black_box(state.clone()), &[]))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
