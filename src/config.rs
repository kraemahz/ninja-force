use amethyst::config::Config;
use serde::{Deserialize, Serialize};

use crate::components::{
    arena::ArenaConfig,
    camera::CameraConfig,
    ground::GroundConfig,
    player::PlayerConfig
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NinjaForceConfig {
    pub arena: ArenaConfig,
    pub camera: CameraConfig,
    pub ground: GroundConfig,
    pub player: PlayerConfig,
}

impl Default for NinjaForceConfig {
    fn default() -> Self {
        Self {
            arena: ArenaConfig::default(),
            camera: CameraConfig::default(),
            player: PlayerConfig::default(),
            ground: GroundConfig::default(),
        }
    }
}
