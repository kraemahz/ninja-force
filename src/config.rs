use serde::{Deserialize, Serialize};

use crate::components::{
    arena::ArenaConfig,
    camera::CameraConfig,
    ground::GroundConfig,
    items::ItemConfig,
    player::PlayerConfig,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NinjaForceConfig {
    pub arena: ArenaConfig,
    pub camera: CameraConfig,
    pub ground: GroundConfig,
    pub items: ItemConfig,
    pub player: PlayerConfig,
}

impl Default for NinjaForceConfig {
    fn default() -> Self {
        Self {
            arena: ArenaConfig::default(),
            camera: CameraConfig::default(),
            ground: GroundConfig::default(),
            items: ItemConfig::default(),
            player: PlayerConfig::default(),
        }
    }
}
