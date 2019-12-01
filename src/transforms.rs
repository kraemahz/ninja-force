use amethyst::{
    assets::Handle,
    core::Transform,
    ecs::World,
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
    window::ScreenDimensions
};

pub const ARENA_HEIGHT: f32 = 200.0;
pub const ARENA_WIDTH: f32 = 200.0;

pub(crate) fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);
    world.create_entity()
         .with(transform)
         .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
         .build();
}
