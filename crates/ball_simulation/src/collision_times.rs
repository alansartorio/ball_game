use nalgebra::Vector2;

use crate::{
    collision_primitives::{ball_ball, segment_ball},
    Ball, Block, Event, WallType,
};

pub(crate) fn earliest_collision_ball_walls(
    ball: &Ball,
    width: f64,
    height: f64,
) -> Option<Event<WallType>> {
    let radius = ball.radius;

    let time_x = if ball.velocity.x > 0.0 {
        Some(Event {
            time: (width - radius - ball.position.x) / ball.velocity.x,
            data: WallType::XPositive,
        })
    } else if ball.velocity.x < 0.0 {
        Some(Event {
            time: (radius - ball.position.x) / ball.velocity.x,
            data: WallType::XNegative,
        })
    } else {
        None
    };

    let time_y = if ball.velocity.y > 0.0 {
        Some(Event {
            time: (height - radius - ball.position.y) / ball.velocity.y,
            data: WallType::YPositive,
        })
    } else if ball.velocity.y < 0.0 {
        Some(Event {
            time: (radius - ball.position.y) / ball.velocity.y,
            data: WallType::YNegative,
        })
    } else {
        None
    };

    time_x
        .into_iter()
        .chain(time_y.into_iter())
        .min_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
}

pub(crate) fn earliest_collision_ball_block(
    ball: &Ball,
    block: &Block,
) -> Option<Event<Vector2<f64>>> {
    let Block {
        min_x,
        max_y,
        max_x,
        min_y,
    } = *block;
    let center = Vector2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    let tl = Vector2::new(min_x, max_y);
    let tr = Vector2::new(max_x, max_y);
    let bl = Vector2::new(min_x, min_y);
    let br = Vector2::new(max_x, min_y);
    let radius = (tl - center).magnitude();
    (ball_ball(
        ball,
        &Ball {
            position: center,
            velocity: Vector2::zeros(),
            radius,
        },
    )
    .is_some()
        || (ball.position - center).magnitude_squared() <= radius.powi(2))
    .then(|| {
        [[tr, tl], [tl, bl], [bl, br], [br, tr]]
            .into_iter()
            .filter_map(|[segment_a, segment_b]| segment_ball(segment_a, segment_b, ball))
            .min_by(|a, b| a.time.total_cmp(&b.time))
    })
    .flatten()
}
