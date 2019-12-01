use amethyst::{
    assets::Loader,
    core::transform::Transform,
    derive::SystemDesc,
    prelude::*,
    ecs::prelude::{Entity, Join, System, SystemData, World, WriteStorage, WriteExpect},
    ui::{Anchor, TtfFormat, UiText, UiTransform}
};

use crate::components::ball::Ball;
use crate::components::paddle::Side;
use crate::transforms::ARENA_WIDTH;

#[derive(SystemDesc)]
pub struct ScoreSystem;


impl<'s> System<'s> for ScoreSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        WriteExpect<'s, ScoreBoard>,
    );

    fn run(&mut self, (mut balls, mut locals, mut ui_text, mut scores): Self::SystemData) {
        for (ball, transform) in (&mut balls, &mut locals).join() {
            let ball_x = transform.translation().x;

            let score_hit: Option<Side> = if ball_x <= ball.radius {
                scores.right_score.add_score(&mut ui_text);
                Some(Side::Right)
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                scores.left_score.add_score(&mut ui_text);
                Some(Side::Left)
            } else {
                None
            };

            if score_hit.is_some() {
                ball.velocity[0] = -ball.velocity[0];
                transform.set_translation_x(ARENA_WIDTH / 2.0);
                println!("{:?} scored", score_hit.unwrap());
            }
        }
    }
}


#[derive(Debug, Default)]
pub struct Score {
    pub score: u32,
    pub display: Option<Entity>
}

impl Score {
    pub fn add_score(&mut self, ui: &mut WriteStorage<'_, UiText>) {
        self.score += 1;
        if let Some(display) = self.display {
            if let Some(ui_display) = ui.get_mut(display) {
                ui_display.text = self.score.to_string();
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ScoreBoard {
    pub left_score: Score,
    pub right_score: Score
}

pub fn initialize_score(world: &mut World) {

    let font = world.read_resource::<Loader>().load(
        "fonts/heavy_data.ttf",
        TtfFormat,
        (),
        &world.read_resource()
    );

    let p1_transform = UiTransform::new(
        "P1".to_string(), Anchor::TopMiddle, Anchor::Middle,
        -50.,
        -50.,
        1.,
        200.,
        50.
    );
    let p2_transform = UiTransform::new(
        "P2".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        50.,
        -50.,
        1.,
        200.,
        50.
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.
        )).build();
    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(
            font.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.
        )).build();

    let mut scores = ScoreBoard::default();
    scores.left_score.display = Some(p1_score);
    scores.right_score.display = Some(p2_score);

    world.insert(scores);
}
