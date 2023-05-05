use nalgebra::Vector2;

use crate::{collision_primitives::segment_ball, Ball, Block, Event, WallType};

pub(crate) fn earliest_collision_ball_walls(
    ball: &Ball,
    width: f64,
    height: f64,
) -> Option<Event<WallType>> {
    let radius = ball.radius;

    let time_x = if ball.velocity.x > 0.0 {
        Some(Event {
            time: (width - radius - ball.position.x) / ball.velocity.x,
            data: WallType::Right,
        })
    } else if ball.velocity.x < 0.0 {
        Some(Event {
            time: (radius - ball.position.x) / ball.velocity.x,
            data: WallType::Left,
        })
    } else {
        None
    };

    let time_y = if ball.velocity.y > 0.0 {
        Some(Event {
            time: (height - radius - ball.position.y) / ball.velocity.y,
            data: WallType::Top,
        })
    } else if ball.velocity.y < 0.0 {
        Some(Event {
            time: (radius - ball.position.y) / ball.velocity.y,
            data: WallType::Bottom,
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
        top,
        left,
        right,
        bottom,
    } = *block;
    let tl = Vector2::new(left, top);
    let tr = Vector2::new(right, top);
    let bl = Vector2::new(left, bottom);
    let br = Vector2::new(right, bottom);
    [[tr, tl], [tl, bl], [bl, br], [br, tr]]
        .into_iter()
        .filter_map(|[segment_a, segment_b]| segment_ball(segment_a, segment_b, ball))
        .min_by(|a, b| a.time.total_cmp(&b.time))
}
