#[macro_use] extern crate amethyst;
use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        rendy
    },
    utils::application_root_dir,
    window::{EventLoop, DisplayConfig}
};
use std::time::Duration;

mod components;
mod state;
mod transforms;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config_dir = app_root.join("config/");
    let display_config_path = config_dir.join("display.ron");

    let assets_dir = app_root.join("assets/");
    let event_loop = EventLoop::new();
    let display_config = DisplayConfig::load(display_config_path).expect("Failed to load DisplayConfig");

    let game_data = GameDataBuilder::default()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new(display_config, &event_loop)
                .with_plugin(RenderToWindow::new()
                    .with_clear(rendy::hal::command::ClearColor{float32: [0.34, 0.36, 0.52, 1.0]})
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(TransformBundle::new())?;

    let mut game = Application::build(assets_dir, state::NinjaForce::new())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();

    Ok(())
}
