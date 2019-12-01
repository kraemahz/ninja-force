use amethyst::{
    assets::Handle,
    core::{Time, Transform, SystemDesc},
    ecs::{Join, Read, ReadStorage, System,
          SystemData, World, WriteStorage,
          Component, DenseVecStorage},
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
};

use crate::components::paddle::{Paddle, Side, PADDLE_HEIGHT, PADDLE_WIDTH};
use crate::transforms::{ARENA_HEIGHT, ARENA_WIDTH};

pub const BALL_VELOCITY_X: f32 = 75.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_RADIUS: f32 = 2.0;

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            velocity: [BALL_VELOCITY_X, BALL_VELOCITY_Y],
            radius: BALL_RADIUS
        }
    }
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialize_ball(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 1
    };

    world.create_entity()
         .with(sprite_render)
         .with(Ball::new())
         .with(transform)
         .build();
}


#[derive(SystemDesc)]
pub struct BallSystem;

impl<'s> System<'s> for BallSystem {
    type SystemData = (
        ReadStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>
    );

    fn run(&mut self, (balls, mut locals, time): Self::SystemData) {
        for (ball, local) in (&balls, &mut locals).join() {
            local.prepend_translation_x(ball.velocity[0] * time.delta_seconds());
            local.prepend_translation_y(ball.velocity[1] * time.delta_seconds());
        }
    }
}

#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>
    );

    fn run(&mut self, (mut balls, paddles, transforms): Self::SystemData) {

        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            let toward_bottom = ball_y <= ball.radius && ball.velocity[1] < 0.0;
            let toward_top = ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0;
            // Handle ball bounce from top and bottom.
            if (toward_bottom || toward_top) {
                ball.velocity[1] = -ball.velocity[1];
            }

            // Handle bound from paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);
                if point_in_rect (
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + paddle.width + ball.radius,
                    paddle_y + paddle.height + ball.radius
                ) {
                    let toward_left = paddle.side == Side::Left && ball.velocity[0] < 0.0;
                    let toward_right = paddle.side == Side::Right && ball.velocity[0] > 0.0;
                    if (toward_left || toward_right) {
                        ball.velocity[0] = -ball.velocity[0];
                    }
                }
            }
        }
    }
}

fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}
