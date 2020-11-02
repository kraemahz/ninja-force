use amethyst::{
    core::{math::Vector2, Transform},
    ecs::{Component, DenseVecStorage, Join, ReadStorage, System, SystemData, World, WriteStorage},
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::geometry::Corners;
use super::physics::{PhysicsBox, InverseBoundingBox2D};

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
        WriteStorage<'s, PhysicsBox>,
        ReadStorage<'s, Arena>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut physics_boxes, arenas, mut transforms): Self::SystemData) {
        for (physics, transform) in (&mut physics_boxes, &mut transforms).join() {
            let position = Vector2::new(transform.translation().x, transform.translation().y);
            let physics_box = physics.bbox.translate(position);
            for arena in (&arenas).join() {
                if let Some(intersection) = arena.inverse_bbox.shortest_manhattan_move(&physics_box)
                {
                    physics.velocity.x = 0.0;
                    physics.velocity.y = 0.0;
                    transform.prepend_translation_x(intersection.x);
                    transform.prepend_translation_y(intersection.y);
                }
            }
        }
    }
}
