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

use crate::components::physics::{accelerate1d, decelerate1d, BoundingBox2D, Point};

pub enum PowerUp {
    KiArmor,
    KiStar,
    KiBlade,
    KiClaws,
    KiFan,
}

const DEFAULT_ACCEL: f32 = 22.0;
const DEFAULT_DECEL: f32 = 26.0;
const MAX_SPEED: f32 = 40.0;

pub struct Player {
    // Set by [InteractiveItemSystem, EnemySystem]
    pub money: i32,
    pub power_up: Option<PowerUp>,

    // Set by GroundSystem
    pub bbox: BoundingBox2D,
    pub max_speed: f32,
    pub on_ground: bool,
    pub ground_accel: f32,
    pub ground_decel: f32,

    // Set by RopeSystem
    pub climbing: bool,

    // Set by PlayerMovementSystem
    pub intent: Point,
    pub running: bool,

    // Set by [PlayerMovementSystem, PlayerPhysicsSystem,
    //         GroundSystem, InteractivePhysicsSystem]
    pub velocity: Point,
}

impl Player {
    pub fn new() -> Self {
        let bbox = BoundingBox2D {corners: [[0.0, 0.0], [16.0, 32.0]]};
        Player {
            money: 0,
            power_up: None,
            bbox,
            max_speed: MAX_SPEED,
            on_ground: true,
            ground_accel: DEFAULT_ACCEL,
            ground_decel: DEFAULT_DECEL,

            climbing: false,

            intent: [0.0, 0.0],
            running: false,

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
        if self.running && self.intent[0].signum() == self.velocity[0].signum() && self.velocity[0] >= (MAX_SPEED * 0.4) {
            self.velocity[1] *= 1.5;
        }
        self.climbing = false;
    }

    pub fn run(&mut self) {
        if self.on_ground {
            self.running = true;
        }
    }

    pub fn reset_frame(&mut self) {
        self.intent = [0.0, 0.0];
        self.running = false;
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
            player.reset_frame();

            if input.action_is_down("run").unwrap_or(false) {
                player.run();
            }
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
                    // If the player is trying to change directions, use the ground decel as
                    // assistance.
                    let base_accel = if player.running { player.ground_accel } else { player.ground_accel / 2.0 };
                    let max_speed = if player.running { MAX_SPEED } else { MAX_SPEED / 2.0 };
                    let accel = if player.intent[0].signum() != player.velocity[0].signum() {
                        base_accel + player.ground_decel
                    } else {
                        base_accel
                    };
                    player.velocity[0] = accelerate1d(
                        player.velocity[0],
                        player.intent[0] * accel,
                        time_step,
                    )
                    .max(-max_speed)
                    .min(max_speed);
                }
            } else {
                player.velocity[1] =
                    accelerate1d(player.velocity[1], -DEFAULT_ACCEL * 2.0, time_step).max(-MAX_SPEED);
            }
        }

        for (player, local) in (&players, &mut locals).join() {
            local.prepend_translation_x(player.velocity[0] * time_step);
            local.prepend_translation_y(player.velocity[1] * time_step);
        }
    }
}
