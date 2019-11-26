use amethyst::prelude::*;

use crate::components::paddle::Paddle;
use crate::loader::load_sprite_sheet;
use crate::transforms::*;

pub struct NinjaForce {}

impl NinjaForce {
    pub fn new() -> Self {
        Self {}
    }
}

impl SimpleState for NinjaForce {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Paddle>();
        let sheet = load_sprite_sheet(world, "pong_spritesheet.png", "pong_spritesheet.ron");

        initialize_paddles(world, sheet);
        initialize_camera(world);
    }
}
