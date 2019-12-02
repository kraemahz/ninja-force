use amethyst::{
    assets::Handle,
    core::{SystemDesc, Time, Transform},
    ecs::{
        Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};

use crate::components::physics::{accelerate1d, decelerate1d, Point};

pub enum PowerUp {
    KiArmor,
    KiStar,
    KiBlade,
    KiClaws,
    KiFan,
}

const DEFAULT_ACCEL: f32 = 12.0;
const DEFAULT_DECEL: f32 = 8.0;
const MAX_SPEED: f32 = 30.0;

pub struct Player {
    // Set by [InteractiveItemSystem, EnemySystem]
    pub money: i32,
    pub power_up: Option<PowerUp>,

    // Set by GroundSystem
    pub max_speed: f32,
    pub on_ground: bool,
    pub ground_accel: f32,
    pub ground_decel: f32,

    // Set by RopeSystem
    pub climbing: bool,

    // Set by PlayerMovementSystem
    pub intent: Point,

    // Set by [PlayerMovementSystem, PlayerPhysicsSystem,
    //         GroundSystem, InteractivePhysicsSystem]
    pub velocity: Point,
}

impl Player {
    pub fn new() -> Self {
        Player {
            money: 0,
            power_up: None,

            max_speed: MAX_SPEED,
            on_ground: true,
            ground_accel: DEFAULT_ACCEL,
            ground_decel: DEFAULT_DECEL,

            climbing: false,

            intent: [0.0, 0.0],

            velocity: [0.0, 0.0],
        }
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.velocity[1] = MAX_SPEED;
        } else if self.climbing {
            self.velocity[0] = self.intent[0] * (MAX_SPEED / 2.0);
            self.velocity[1] = MAX_SPEED / 2.0;
        }
        self.climbing = false;
        self.on_ground = false;
    }

    pub fn climb(&mut self) {
        self.on_ground = false;
        self.climbing = true;
        self.velocity = [0.0, 0.0];
    }

    /// Damage the player. Returns false if the Player is dead.
    pub fn damage(&mut self) -> bool {
        match self.power_up.take() {
            Some(power_up) => {
                match power_up {
                    PowerUp::KiArmor => {
                        self.power_up = None;
                    }
                    _ => {
                        self.power_up = Some(PowerUp::KiArmor);
                    }
                }
                true
            }
            None => false,
        }
    }

    pub fn collect(&mut self, power_up: PowerUp) {
        match power_up {
            PowerUp::KiArmor => {
                if self.power_up.is_none() {
                    self.power_up = Some(power_up)
                }
            }
            _ => {
                self.power_up = Some(power_up);
            }
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

pub fn initialize_player(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet,
        sprite_number: 0,
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(16.0, 24.0, 0.0);
    world
        .create_entity()
        .with(sprite_render)
        .with(Player::new())
        .with(transform)
        .build();
}

pub struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut players, input): Self::SystemData) {
        for (player,) in (&mut players,).join() {
            player.intent = [0.0, 0.0];
            if let Some(mv_x_axis) = input.axis_value("x") {
                player.intent[0] = mv_x_axis;
            }
            if let Some(mv_y_axis) = input.axis_value("y") {
                player.intent[1] = mv_y_axis;
            }
            if input.action_is_down("jump").unwrap_or(false) {
                player.jump();
            }
        }
    }
}

pub struct PlayerPhysicsSystem;

impl<'s> System<'s> for PlayerPhysicsSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut players, mut locals, time): Self::SystemData) {
        let time_step = time.delta_seconds();
        for (player,) in (&mut players,).join() {
            if player.on_ground {
                if player.intent[0] == 0.0 {
                    player.velocity[0] =
                        decelerate1d(player.velocity[0], player.ground_decel, time_step);
                } else {
                    player.velocity[0] = accelerate1d(
                        player.velocity[0],
                        player.intent[0] * player.ground_accel,
                        time_step,
                    )
                    .max(-MAX_SPEED)
                    .min(MAX_SPEED);
                }
            } else {
                player.velocity[1] =
                    accelerate1d(player.velocity[1], -DEFAULT_ACCEL, time_step).max(-MAX_SPEED);
            }
        }

        for (player, local) in (&players, &mut locals).join() {
            local.prepend_translation_x(player.velocity[0] * time_step);
            local.prepend_translation_y(player.velocity[1] * time_step);
        }
    }
}
