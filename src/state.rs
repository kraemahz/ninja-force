use amethyst::{
    animation::AnimationSetPrefab,
    assets::{AssetStorage, Handle, Loader,
             Prefab, PrefabData, PrefabLoader,
             ProgressCounter, RonFormat},
    core::Time,
    ecs::prelude::Entity,
    error::Error,
    prelude::*,
    renderer::{
        formats::texture::ImageFormat,
        sprite::{SpriteSheet, SpriteSheetFormat, SpriteRender, prefab::SpriteScenePrefab},
        Texture,
    },
};
use serde::{Deserialize, Serialize};

use crate::components::ground::initialize_ground;
use crate::components::player::initialize_player;
use crate::transforms::*;

/// Animation ids used in a AnimationSet
#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    Move,
}

/// Loading data for one entity
#[derive(Debug, Clone, Deserialize, PrefabData)]
pub struct SpritePrefab {
    /// Information for rendering a scene with sprites
    sprite_scene: SpriteScenePrefab,
    /// Аll animations that can be run on the entity
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}

pub struct NinjaForce {
    pub progress_counter: Option<ProgressCounter>,
}

impl NinjaForce {
    pub fn new() -> Self {
        Self {progress_counter: None}
    }

    pub fn load_sprite_sheet(
        &mut self,
        world: &mut World,
        sprite: &str,
        sprite_sheet: &str,
    ) -> Handle<SpriteSheet> {
        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                sprite,
                ImageFormat::default(), 
                self.progress_counter.as_mut().unwrap(),
                &texture_storage)
        };
        let loader = world.read_resource::<Loader>();
        let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

        loader.load(
            sprite_sheet,
            SpriteSheetFormat(texture_handle),
            self.progress_counter.as_mut().unwrap(),
            &sprite_sheet_store,
        )
    }

    pub fn load_prefab(
        &mut self,
        world: &mut World,
        prefab: &str,
    ) -> Handle<Prefab<SpritePrefab>> {
        world.exec(|loader: PrefabLoader<'_, SpritePrefab>| {
            loader.load(
                prefab,
                RonFormat,
                self.progress_counter.as_mut().unwrap(),
            )
        })
    }
}

impl SimpleState for NinjaForce {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.progress_counter = Some(Default::default());
        let ground_sprite = self.load_sprite_sheet(
            world,
            "sprites/dirt.png",
            "sprites/dirt.ron"
        );
        let player_sprite = self.load_sprite_sheet(
            world,
            "sprites/player.png",
            "sprites/player.ron"
        );

        initialize_ground(world, ground_sprite);
        initialize_player(world, player_sprite);
        initialize_camera(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
}
