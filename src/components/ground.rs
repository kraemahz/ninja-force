use amethyst::{
    assets::Handle,
    core::{SystemDesc, Time, Transform},
    ecs::{
        Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};

use crate::components::physics::{BoundingBox2D, Point};
use crate::components::player::Player;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

pub struct Ground {
    pub bbox: BoundingBox2D,
}

impl Ground {
    pub fn new(corner: Point) -> Self {
        Self {
            bbox: BoundingBox2D {
                corners: [corner, [corner[0] + 16.0, corner[1] + 24.0]],
            },
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
            sprite_number: between.sample(&mut rng),
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz((16 * index) as f32, 0.0, 0.0);

        world
            .create_entity()
            .with(sprite_render)
            .with(Ground::new([(16 * index) as f32, 0.0]))
            .with(transform)
            .build();
    }

    for index in 8..16 {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: between.sample(&mut rng),
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz((16 * index) as f32, 32.0, 0.0);

        world
            .create_entity()
            .with(sprite_render)
            .with(Ground::new([(16 * index) as f32, 32.0]))
            .with(transform)
            .build();
    }

    let between = Uniform::from(20..30);

    for index in 1..8 {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: between.sample(&mut rng),
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz(16.0 * 12.0, 32.0 + (16 * index) as f32, 0.0);

        world
            .create_entity()
            .with(sprite_render)
            .with(Ground::new([16.0 * 12.0, 32.0 + (16 * index) as f32]))
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
            player.on_ground = false;
            let player_position = [transform.translation().x, transform.translation().y];
            let player_box = player.bbox.translate(player_position);
            for (ground, ground_transform) in (&grounds, &transforms).join() {
                if ground.bbox.intersects(&player_box) {
                    player.velocity[1] = 0.0;
                    player.on_ground = true;
                }
            }
        }
    }
}
