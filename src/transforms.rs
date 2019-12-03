use amethyst::{
    assets::Handle,
    core::Transform,
    ecs::World,
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
    window::ScreenDimensions,
};
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct ArenaConfig {
    pub width: f32,
    pub height: f32
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 200.0
        }
    }
}

pub(crate) fn initialize_camera(world: &mut World) {
    let (arena_height, arena_width) = {
        let config = world.read_resource::<ArenaConfig>();
        (config.height, config.width)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(arena_width * 0.5, arena_height * 0.5, 1.0);
    world
        .create_entity()
        .with(transform)
        .with(Camera::standard_2d(arena_width, arena_height))
        .build();
}
