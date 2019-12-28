use amethyst::{
    assets::Handle,
    core::{math::Vector3, Transform},
    ecs::{Join, Read, ReadStorage, System,
          SystemData, World, WriteStorage,
          Component, DenseVecStorage},
    prelude::*,
    renderer::{Camera, SpriteRender, SpriteSheet},
    window::ScreenDimensions,
};
use serde::{Deserialize, Serialize};
use super::player::Player;

#[derive(Debug, Deserialize, Serialize)]
pub struct CameraConfig {
    pub width: f32,
    pub height: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 200.0,
        }
    }
}

pub(crate) fn initialize_camera(world: &mut World) {
    let (screen_height, screen_width) = {
        let config = world.read_resource::<CameraConfig>();
        (config.height, config.width)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(screen_width * 0.5, screen_height * 0.5, 1.0);
    world
        .create_entity()
        .with(transform)
        .with(Camera::standard_2d(screen_width, screen_height))
        .build();
}


pub struct CameraMovementSystem;
impl<'s> System<'s> for CameraMovementSystem {
    type SystemData = (
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
    );
    fn run(&mut self, (cameras, players, mut transforms): Self::SystemData) {
        let get_camera_translation = || -> Option<Vector3<f32>> {
            for (_player, player_transform) in (&players, &transforms).join() {
                let player_vec = player_transform.translation();
                let camera_vec = Vector3::new(player_vec.x, player_vec.y, 1.0);
                return Some(camera_vec);
            }
            None
        };
        let new_translation = get_camera_translation().unwrap();

        for (_camera, transform) in (&cameras, &mut transforms).join() {
            transform.set_translation(new_translation);
        }
    }
}
