use amethyst::{
    assets::Handle,
    core::Transform,
    ecs::World,
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
    window::ScreenDimensions
};

use crate::components::paddle::*;

fn get_camera_dims(world: &mut World) -> (f32, f32) {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
}

pub(crate) fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    let (width, height) = get_camera_dims(world);
    transform.set_translation_z(1.0);
    world.create_entity()
         .with(transform)
         .with(Camera::standard_2d(width, height))
         .build();
}

pub fn initialize_paddles(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    let (width, height) = get_camera_dims(world);
    let y = height / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(width - PADDLE_WIDTH * 0.5, y, 0.0);

    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .build();

    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .build();
}
