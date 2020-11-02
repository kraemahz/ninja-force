use amethyst::{
    assets::Loader,
    derive::SystemDesc,
    prelude::*,
    ecs::prelude::{Join, System, SystemData, World, ReadStorage, WriteStorage},
    ui::{Anchor, TtfFormat, UiText, UiTransform}
};

use super::player::Player;

#[derive(SystemDesc)]
pub struct ScoreSystem;

impl<'s> System<'s> for ScoreSystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        ReadStorage<'s, Player>,
    );

    fn run(&mut self, (mut ui_text, players): Self::SystemData) {
        for ui_text in (&mut ui_text).join() {
            for player in (&players).join() {
                ui_text.text = player.score.to_string();
            }
        }
    }
}

pub fn initialize_score(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "fonts/heavy_data.ttf",
        TtfFormat,
        (),
        &world.read_resource()
    );

    let transform = UiTransform::new(
        "Score".to_string(),
        Anchor::TopRight,
        Anchor::MiddleLeft,
        50., -50., 1., 200., 50.
    );

    world.create_entity()
         .with(transform)
         .with(UiText::new(font.clone(),
                           "0".to_string(),
                           [1., 1., 1., 1.],
                           50.))
         .build();
}
