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
use serde::{Deserialize, Serialize};

use crate::components::physics::{BoundingBox2D, Vector2};
use crate::components::player::Player;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

pub struct Ground {
    pub bbox: BoundingBox2D,
}

impl Ground {
    pub fn new(corner: Vector2) -> Self {
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


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroundPosition {
    pub num: usize,
    pub pos: Vector2
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroundConfig {
    pub elements: Vec<GroundPosition>
}

impl Default for GroundConfig {
    fn default() -> Self {
        Self{
            elements: Vec::new()
        }
    }
}

pub fn initialize_ground(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let elements: Vec<GroundPosition> = {
        let config = world.read_resource::<GroundConfig>();
        config.elements.iter().cloned().collect()
    };

    for elem in elements {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: elem.num
        };

        let mut transform = Transform::default();
        transform.set_translation_xyz(elem.pos[0], elem.pos[1], 0.0);

        world
            .create_entity()
            .with(sprite_render)
            .with(Ground::new(elem.pos))
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
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, grounds, mut transforms): Self::SystemData) {
        for (player, transform) in (&mut players, &mut transforms).join() {
            player.on_ground = false;
            let player_position = [transform.translation().x, transform.translation().y];
            let player_box = player.bbox.translate(player_position);
            for (ground,) in (&grounds,).join() {
                if let Some(intersection) = ground.bbox.shortest_manhattan_move(&player_box) {
                    // Hit the ground from the top
                    if intersection[1] > 0.0 {
                        player.on_ground = true;
                        transform.prepend_translation_y(intersection[1]);
                        player.velocity[1] = 0.0;
                    // Hit the ceiling from the bottom
                    } else if intersection[1] < 0.0 {
                        transform.prepend_translation_y(intersection[1]);
                        player.velocity[1] = -player.velocity[1]; 
                    // Hit a wall.
                    } else if intersection[0] != 0.0 {
                        player.velocity[0] = -player.velocity[0] / 2.0;
                        transform.prepend_translation_x(intersection[0]);
                    }
                }
            }
        }
    }
}
