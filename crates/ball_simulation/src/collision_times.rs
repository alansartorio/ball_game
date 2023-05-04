use crate::{Ball, Block, Collision, WallType};

pub(crate) fn earliest_collision_ball_walls(
    ball: &Ball,
    width: f64,
    height: f64,
) -> Option<Collision<WallType>> {
    let radius = ball.radius;

    let time_x = if ball.velocity.x > 0.0 {
        Some((width - radius - ball.position.x) / ball.velocity.x)
    } else if ball.velocity.x < 0.0 {
        Some((radius - ball.position.x) / ball.velocity.x)
    } else {
        None
    }
    .map(|time| Collision {
        time,
        data: WallType::Vertical,
    });

    let time_y = if ball.velocity.y > 0.0 {
        Some((height - radius - ball.position.y) / ball.velocity.y)
    } else if ball.velocity.y < 0.0 {
        Some((radius - ball.position.y) / ball.velocity.y)
    } else {
        None
    }
    .map(|time| Collision {
        time,
        data: WallType::Horizontal,
    });

    time_x
        .into_iter()
        .chain(time_y.into_iter())
        .min_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
}

pub(crate) fn earliest_collision_ball_block(ball: &Ball, block: &Block) -> Option<Collision<()>> {
    todo!();
}
