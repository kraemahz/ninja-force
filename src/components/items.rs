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

use crate::components::physics::{BoundingBox2D, Corners};
use crate::components::player::{PowerUp, Player, PlayerStance};


#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum ItemKind {
    Background,
    Climbable,
    Collectable(PowerUp)
}

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub bbox: BoundingBox2D,
}


impl Item {
    pub fn new(kind: ItemKind, corners: Corners) -> Self {
        Self {
            kind,
            bbox: BoundingBox2D {corners}
        }
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
            .with(Item::new(elem.kind, elem.corners))
            .with(transform)
            .build();
    }
}


#[derive(SystemDesc)]
pub struct InteractableItemSystem;

impl<'s> System<'s> for InteractableItemSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        Entities<'s>,
        ReadStorage<'s, Item>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, entities, items, transforms): Self::SystemData) {
        for (player, transform) in (&mut players, &transforms).join() {
            let player_position =
                Vector2::new(transform.translation().x, transform.translation().y);
            let player_box = player.bbox.translate(player_position);

            let mut on_climbable = false;
            for (entity, item) in (&*entities, &items).join() {
                if item.bbox.intersects(&player_box) {
                    match item.kind {
                        ItemKind::Climbable => {
                            on_climbable = true;
                            if player.intent.y > 0.0 && player.stance != PlayerStance::Climbing {
                                player.climb();
                            }
                        },
                        ItemKind::Collectable(power_up) => {
                            player.collect(power_up);
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
