use amethyst::{
    assets::Handle,
    core::{math::Vector2, Transform},
    ecs::{
        Component, DenseVecStorage, Join,  ReadStorage, System, SystemData, World,
        WriteStorage,
    },
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use serde::{Deserialize, Serialize};

use crate::components::physics::{BoundingBox2D, Corners};
use crate::components::player::{Player, PlayerStance};

pub struct Ground {
    pub bbox: BoundingBox2D,
}

impl Ground {
    pub fn new(corner: Vector2<f32>) -> Self {
        Self {
            bbox: BoundingBox2D {
                corners: Corners {
                    bottom_left: corner,
                    top_right: Vector2::new(corner.x + 16.0, corner.y + 24.0),
                },
            },
        }
    }
}

impl Component for Ground {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroundPosition {
    pub sprite_num: usize,
    pub pos: Vector2<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroundConfig {
    pub elements: Vec<GroundPosition>,
}

impl Default for GroundConfig {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

pub fn initialize_ground(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let elements: Vec<GroundPosition> = {
        let config = world.read_resource::<GroundConfig>();
        config.elements.clone()
    };

    for elem in elements {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: elem.sprite_num,
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

fn compute_intersection_force(intersection: Vector2<f32>, transform: &mut Transform, player: &mut Player) {
    // Hit the ground from the top
    if intersection.y > 0.0 {
        player.on_ground = true;
        transform.prepend_translation_y(intersection.y);
        // Ground should always be at an integer position.
        let y_pos = transform.translation().y;
        transform.prepend_translation_y(-(y_pos - y_pos.round()));
        // Don't stop upwards motion.
        player.velocity.y = player.velocity.y.max(0.0);
    // Hit the ceiling from the bottom
    } else if intersection.y < 0.0 {
        println!("hit bottom: {:?} {:?}", intersection, player.velocity);
        transform.prepend_translation_y(intersection.y);
        player.velocity.y = -player.velocity.y;
    // Hit a wall.
    } else if intersection.x != 0.0 {
        player.velocity.x = -player.velocity.x / 2.0;
        transform.prepend_translation_x(intersection.x);
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
            player.blocked = false;

            let player_position =
                Vector2::new(transform.translation().x, transform.translation().y);
            let player_box = player.bbox.translate(player_position);

            let point_above = Vector2::new(player_box.corners.x_midpoint(),
                                           player_box.corners.top() + 1.0);
            let point_above_crouching = Vector2::new(
                player_box.corners.x_midpoint(),
                player_box.corners.bottom() + 9.0
            );
            let box_below = player_box.translate(Vector2::new(0.0, -0.5));

            let mut intersections = Vec::new();
            let mut intersection_below = false;
            let mut intersection_crouched = false;
            let mut intersection_above = false;

            for ground in (&grounds).join() {
                let intersection_check = ground
                    .bbox
                    .shortest_manhattan_move(&player_box, player.velocity);

                if let Some(intersection) = intersection_check {
                    println!("intersection: {:?}", -intersection);
                    intersections.push(-intersection);
                }

                // Fall test. If the player moves down will they intersect the ground? If so they are
                // on the ground. If not, they will fall.
                intersection_below |= ground.bbox.intersects(&box_below);
                intersection_crouched |= ground.bbox.contains(point_above_crouching);
                intersection_above |= ground.bbox.contains(point_above);
            }
            player.on_ground = intersection_below;

            if intersection_below && intersection_above && (intersection_crouched || intersections.len() > 0) {
                // The player is in a cramped space and colliding with an object. Force crouching.
                player.stance = PlayerStance::Crouching;
                player.blocked = true;
                break;
            }

            let mut minimum_intersection: Option<Vector2<f32>> = None;
            for intersection in intersections {
                if let Some(min_intersection) = minimum_intersection {
                    let int_sum = intersection.x.abs() + intersection.y.abs();
                    let min_int_sum = min_intersection.x.abs() + min_intersection.y.abs();
                    if int_sum < min_int_sum {
                        minimum_intersection = Some(intersection);
                    }
                } else {
                    minimum_intersection = Some(intersection);
                }
            }

            if let Some(intersection) = minimum_intersection {
                compute_intersection_force(intersection, transform, player);
            }
        }
    }
}
