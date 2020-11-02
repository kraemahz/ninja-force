use amethyst::{
    assets::Handle,
    core::{math::Vector2, Time, Transform},
    ecs::{Component, DenseVecStorage, Join, Read, ReadStorage, System, World, WriteStorage},
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use serde::{Deserialize, Serialize};

use crate::components::physics::{
    accelerate1d, decelerate1d, BoundingBox2D, Corners, MINIMUM_CLIP,
};


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EnemyKind {
    Grunt
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnemyConfig {
    location: Vector2<f32>,
    kind: EnemyKind,
    max_speed: f32,
    accel: f32,
    fall_accel: f32,
}
