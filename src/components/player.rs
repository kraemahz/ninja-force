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
use serde::{Deserialize, Serialize};

use crate::components::physics::{accelerate1d, decelerate1d, BoundingBox2D, Vector2};

pub enum PowerUp {
    KiArmor,
    KiStar,
    KiBlade,
    KiClaws,
    KiFan,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub accel_running: f32,
    pub accel_walking: f32,
    pub accel_climbing: f32,
    pub decel_ground: f32,
    pub decel_climbing: f32,
    pub max_speed_walking: f32,
    pub max_speed_running: f32,
    pub max_speed_climbing: f32,
    pub max_speed_falling: f32,
    pub min_running_jump_speed: f32,
    pub jump_speed_walking: f32,
    pub jump_speed_running: f32,
    pub jump_speed_climbing: f32,
    pub fall_accel: f32,
}


impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            accel_running: 22.0,
            accel_walking: 11.0,
            accel_climbing: 11.0,
            decel_ground: 26.0,
            decel_climbing: 32.0,
            max_speed_walking: 20.0,
            max_speed_running: 40.0,
            max_speed_climbing: 20.0,
            max_speed_falling: 40.0,
            min_running_jump_speed: 20.5,
            jump_speed_walking: 60.0,
            jump_speed_running: 90.0,
            jump_speed_climbing: 60.0,
            fall_accel: 22.0,
        }
    }
}


pub struct Player {
    pub config: PlayerConfig,

    // Set by [InteractiveItemSystem, EnemySystem]
    pub money: i32,
    pub power_up: Option<PowerUp>,

    // Set by GroundSystem
    pub bbox: BoundingBox2D,
    pub on_ground: bool,

    // Set by RopeSystem
    pub climbing: bool,

    // Set by PlayerMovementSystem
    pub intent: Vector2,
    pub running: bool,

    // Set by [PlayerMovementSystem, PlayerPhysicsSystem,
    //         GroundSystem, InteractivePhysicsSystem]
    pub velocity: Vector2,
}

impl Player {
    pub fn new(config: PlayerConfig) -> Self {
        let bbox = BoundingBox2D {corners: [[4.0, 0.0], [12.0, 32.0]]};
        Player {
            config,
            money: 0,
            power_up: None,
            bbox,
            on_ground: true,
            climbing: false,
            intent: [0.0, 0.0],
            running: false,
            velocity: [0.0, 0.0],
        }
    }

    pub fn jump(&mut self) {
        if self.running 
                && self.intent[0].signum() == self.velocity[0].signum()
                && self.velocity[0].abs() >= self.config.min_running_jump_speed {
            self.velocity[1] = self.config.jump_speed_running;
        } else if self.on_ground {
            self.velocity[1] = self.config.jump_speed_walking;
        } else if self.climbing {
            self.climbing = false;
            self.velocity[0] = self.intent[0] * self.config.accel_climbing;
            self.velocity[1] = self.config.jump_speed_climbing;
        }
    }

    pub fn fall(&mut self, time_step: f32) {
        self.velocity[1] =
            accelerate1d(self.velocity[1], -self.config.fall_accel, time_step)
            .max(-self.config.max_speed_falling);
    }

    pub fn ground_slide(&mut self, time_step: f32) {
        self.velocity[0] =
            decelerate1d(self.velocity[0], self.config.decel_ground, time_step);
    }

    pub fn ground_move(&mut self, time_step: f32) {
        // If the player is trying to change directions, use the ground decel as
        // assistance.
        let base_accel = if self.running { self.config.accel_running } else { self.config.accel_walking };
        let max_speed = if self.running { self.config.max_speed_running } else { self.config.max_speed_walking };
        let accel = if self.intent[0].signum() != self.velocity[0].signum() {
            base_accel + self.config.decel_ground
        } else {
            base_accel
        };
        self.velocity[0] = accelerate1d(
            self.velocity[0],
            self.intent[0] * accel,
            time_step,
        )
        .max(-max_speed)
        .min(max_speed);
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

pub fn initialize_player(world: &mut World,
                         sprite_sheet: Handle<SpriteSheet>,
                         player_start: Vector2) {
    let config = *world.read_resource::<PlayerConfig>();

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet,
        sprite_number: 0,
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(player_start[0], player_start[1], 0.0);
    world
        .create_entity()
        .with(sprite_render)
        .with(Player::new(config))
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
                    player.ground_slide(time_step);
                } else {
                    player.ground_move(time_step);
                }
            } else {
                player.fall(time_step);
            }
        }

        for (player, local) in (&players, &mut locals).join() {
            local.prepend_translation_x(player.velocity[0] * time_step);
            local.prepend_translation_y(player.velocity[1] * time_step);
        }
    }
}
