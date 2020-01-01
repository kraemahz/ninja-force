use amethyst::{
    assets::Handle,
    core::{math::Vector2, SystemDesc, Time, Transform},
    ecs::{Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, World, WriteStorage},
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};

use crate::components::physics::{Corners, InverseBoundingBox2D};
use crate::components::player::Player;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArenaConfig {
    pub corners: Corners,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            corners: Corners {
                bottom_left: Vector2::new(0.0, 0.0),
                top_right: Vector2::new(200.0, 200.0),
            },
        }
    }
}

pub struct Arena {
    pub inverse_bbox: InverseBoundingBox2D,
}

impl Arena {
    pub fn new(corners: Corners) -> Self {
        Self {
            inverse_bbox: InverseBoundingBox2D { corners },
        }
    }
}

impl Component for Arena {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialize_arena(world: &mut World) {
    let corners = {
        let conf = world.read_resource::<ArenaConfig>();
        conf.corners
    };
    world.create_entity().with(Arena::new(corners)).build();
}

#[derive(SystemDesc)]
pub struct ArenaSystem;

impl<'s> System<'s> for ArenaSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Arena>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, arenas, mut transforms): Self::SystemData) {
        for (player, transform) in (&mut players, &mut transforms).join() {
            let player_position =
                Vector2::new(transform.translation().x, transform.translation().y);
            let player_box = player.bbox.translate(player_position);
            for (arena,) in (&arenas,).join() {
                if let Some(intersection) = arena.inverse_bbox.shortest_manhattan_move(&player_box)
                {
                    player.velocity.x = 0.0;
                    player.velocity.y = 0.0;
                    transform.prepend_translation_x(intersection.x);
                    transform.prepend_translation_y(intersection.y);
                }
            }
        }
    }
}
