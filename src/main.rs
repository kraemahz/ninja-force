#[macro_use]
extern crate amethyst;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use amethyst::{
    config::Config,
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};
use std::time::Duration;

mod components;
mod config;
mod state;

use crate::config::NinjaForceConfig;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config_dir = app_root.join("config/");
    let assets_dir = app_root.join("assets/");
    let binding_path = config_dir.join("bindings.ron");
    let display_config_path = config_dir.join("display.ron");
    let game_config = NinjaForceConfig::load(config_dir.join("game.ron"))?;

    let input_bundle =
        InputBundle::<StringBindings>::new().with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(
            components::player::PlayerMovementSystem,
            "movement_system",
            &["input_system"],
        )
        .with(components::arena::ArenaSystem, "arena_system", &[])
        .with(
            components::camera::CameraMovementSystem,
            "camera_system",
            &[],
        )
        .with(components::ground::GroundSystem, "ground_system", &[])
        .with(components::items::InteractableItemSystem, "item_system", &["movement_system"])
        .with(
            components::player::PlayerPhysicsSystem,
            "player_physics_system",
            &["ground_system", "movement_system"],
        )
        .with(
            components::player::PlayerSpriteSystem,
            "player_sprite_system",
            &["player_physics_system"],
        )
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?;

    let mut game = Application::build(assets_dir, state::NinjaForce::new())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_resource(game_config.arena)
        .with_resource(game_config.camera)
        .with_resource(game_config.items)
        .with_resource(game_config.ground)
        .with_resource(game_config.player)
        .build(game_data)?;
    game.run();

    Ok(())
}
