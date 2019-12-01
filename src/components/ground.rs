use amethyst::{
    assets::Handle,
    core::{Time, Transform, SystemDesc},
    ecs::{Join, Read, ReadStorage, System,
          SystemData, World, WriteStorage,
          Component, DenseVecStorage},
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};

use rand::{thread_rng, distributions::{Distribution, Uniform}};
use crate::components::physics::{BoundingBox2D, Point};
use crate::components::player::Player;


pub struct Ground {
    pub bbox: BoundingBox2D
}

impl Ground {
    pub fn new(corner: Point) -> Self {
        Self {
            bbox: BoundingBox2D {
                corners: [
                    corner,
                    [corner[0] + 16.0, corner[1] + 16.0]
                ]
            }
        }
    }
}

impl Component for Ground {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialize_ground(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut rng = thread_rng();
    let between = Uniform::from(0..10);

    for index in 0..16 {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: between.sample(&mut rng)
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0 + (16 * index) as f32, 0.0, 0.0);

        world.create_entity()
             .with(sprite_render)
             .with(Ground::new([0.0 + (16 * index) as f32, 0.0]))
             .with(transform)
             .build();
    }
}

#[derive(SystemDesc)]
pub struct GroundSystem;

impl<'s> System<'s> for GroundSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Ground>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, grounds, transforms): Self::SystemData) {
        for (player, transform) in (&mut players, &transforms).join() {
            let player_position = [transform.translation().x,
                                   transform.translation().y];
            for (ground, ground_transform) in (&grounds, &transforms).join() {
                if ground.bbox.contains(player_position) {
                    player.velocity[1] = 0.0;
                }
            }
        }
    }
}
