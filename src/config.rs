use amethyst::config::Config;
use serde::{Deserialize, Serialize};

use crate::components::ground::GroundConfig;
use crate::components::player::PlayerConfig;
use crate::transforms::ArenaConfig;


#[derive(Debug, Serialize, Deserialize)]
pub struct NinjaForceConfig {
    pub player: PlayerConfig,
    pub ground: GroundConfig,
    pub arena: ArenaConfig,
}

impl Default for NinjaForceConfig {
    fn default() -> Self {
        Self {
            player: PlayerConfig::default(),
            ground: GroundConfig::default(),
            arena: ArenaConfig::default()
        }
    }
}
