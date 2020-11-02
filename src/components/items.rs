use amethyst::{
    assets::Handle,
    core::{math::Vector2, Transform},
    ecs::{prelude::*,
          Component,
          DenseVecStorage,
          Join,
          ReadStorage,
          System,
          SystemData,
          World,
          WriteStorage},
    renderer::{SpriteRender, SpriteSheet},
};
use serde::{Deserialize, Serialize};

use super::physics::{BoundingBox2D, PhysicsBox};
use super::player::{PowerUp, Player, PlayerStance};
use crate::geometry::Corners;


#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum ItemKind {
    Background,
    Climbable,
    Collectable(PowerUp)
}

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub value: usize,
}


impl Item {
    pub fn new(kind: ItemKind) -> Self {
        Self { kind, value: 1000 }
    }
}


impl Component for Item {
    type Storage = DenseVecStorage<Self>;
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemPosition {
    pub sprite_num: usize,
    pub kind: ItemKind,
    pub position: Vector2<f32>,
    pub corners: Corners,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemConfig {
    pub elements: Vec<ItemPosition>,
}

impl Default for ItemConfig {
    fn default() -> Self {
        Self {elements: Vec::new()}
    }
}


const ITEM_PLANE: f32 = -0.001;


pub fn initialize_items(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let elements: Vec<ItemPosition> = {
        let config = world.read_resource::<ItemConfig>();
        config.elements.clone()
    };

    for elem in elements {
        let sprite_render = SpriteRender {
            sprite_sheet: sprite_sheet.clone(),
            sprite_number: elem.sprite_num
        };
        let mut transform = Transform::default();
        transform.set_translation_xyz(elem.position.x, elem.position.y, ITEM_PLANE);

        world.create_entity()
            .with(sprite_render)
            .with(Item::new(elem.kind))
            .with(PhysicsBox::new(BoundingBox2D{corners: elem.corners}))
            .with(transform)
            .build();
    }
}


#[derive(SystemDesc)]
pub struct InteractableItemSystem;

impl<'s> System<'s> for InteractableItemSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, PhysicsBox>,
        Entities<'s>,
        ReadStorage<'s, Item>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, mut physics_boxes, entities, items, transforms): Self::SystemData) {
        let mut player_boxes: Vec<(&mut Player, BoundingBox2D)> = Vec::new();
        for (player, physics, transform) in (&mut players, &physics_boxes, &transforms).join() {
            let player_position =
                Vector2::new(transform.translation().x, transform.translation().y);
            let player_box = physics.bbox.translate(player_position);
            player_boxes.push((player, player_box));
        }

        for (player, player_box) in player_boxes {
            let mut on_climbable = false;
            for (entity, item_physics, item) in (&*entities, &physics_boxes, &items).join() {
                if item_physics.bbox.intersects(&player_box) {
                    match item.kind {
                        ItemKind::Climbable => {
                            on_climbable = true;
                            if player.intent.y > 0.0 && player.stance != PlayerStance::Climbing {
                                player.climb();
                            }
                        },
                        ItemKind::Collectable(power_up) => {
                            player.collect(power_up);
                            player.score += item.value;
                            entities.delete(entity).ok();
                        },
                        _ => {}
                    }

                }
            }
            if !on_climbable && player.stance == PlayerStance::Climbing {
                player.stance = PlayerStance::Standing;
            }
        }
    } 
}
