use amethyst::{
    assets::Handle,
    core::{math::Vector2, Time, Transform},
    ecs::{Component, DenseVecStorage, Join, Read, ReadStorage, System, World, Write, WriteStorage},
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use serde::{Deserialize, Serialize};

use crate::geometry::Corners;
use super::physics::{
    accelerate1d, decelerate1d, BoundingBox2D, PhysicsBox, MINIMUM_CLIP,
};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
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

#[derive(Debug, PartialEq)]
pub enum PlayerAnimationState {
    Standing,
    Walking,
    Crouching,
    Running,
    Stopping,
    Jumping,
    Falling,
    Damaged,
    Dying,
    Climbing,
}

#[derive(Debug, PartialEq)]
pub enum PlayerStance {
    Standing,
    Crouching,
    Climbing
}

#[derive(Debug)]
pub struct Player {
    pub config: PlayerConfig,
    pub game_counter: u32,
    pub animation_state: PlayerAnimationState,

    // Set by [InteractiveItemSystem, EnemySystem]
    pub money: usize,
    pub score: usize,
    pub power_up: Option<PowerUp>,

    // Set by GroundSystem
    pub stance: PlayerStance,
    pub on_ground: bool,
    pub blocked: bool,

    // Set by PlayerMovementSystem
    pub intent: Vector2<f32>,
    pub running: bool,
    pub jumping: bool,
    pub jump_edge: bool,
}

lazy_static! {
    static ref STANDING_BBOX: BoundingBox2D = {
        BoundingBox2D {
            corners: Corners {
                bottom_left: Vector2::new(4.0, 0.0),
                top_right: Vector2::new(12.0, 15.5),
            },
        }
    };
}

lazy_static! {
    static ref CROUCHING_BBOX: BoundingBox2D = {
        BoundingBox2D {
            corners: Corners {
                bottom_left: Vector2::new(4.0, 0.0),
                top_right: Vector2::new(12.0, 7.5),
            },
        }
    };
}

impl Default for Player {
    fn default() -> Self {
        Player::new(PlayerConfig::default())
    }
}

impl Player {
    pub fn new(config: PlayerConfig) -> Self {
        Player {
            config,
            game_counter: 0,
            animation_state: PlayerAnimationState::Standing,
            money: 0,
            score: 0,
            power_up: None,
            stance: PlayerStance::Standing,
            on_ground: true,
            blocked: false,
            intent: Vector2::new(0.0, 0.0),
            running: false,
            jumping: false,
            jump_edge: false,
        }
    }

    pub fn maybe_jump(&mut self, physics: &mut PhysicsBox) {
        // Don't jump again if the player is holding jump.
        if !self.jump_edge || self.blocked {
            return;
        }
        self.animation_state = PlayerAnimationState::Jumping;
        self.jumping = true;
        if self.running
            && self.intent.x.signum() == physics.velocity.x.signum()
            && physics.velocity.x.abs() >= self.config.min_running_jump_speed
        {
            physics.velocity.y = self.config.jump_speed_running;
        } else if self.on_ground {
            physics.velocity.y = self.config.jump_speed_walking;
        } else if self.stance == PlayerStance::Climbing {
            self.stance = PlayerStance::Standing;
            physics.velocity.x = self.intent.x * self.config.accel_climbing;
            physics.velocity.y = self.config.jump_speed_climbing;
        }
    }

    pub fn fall(&mut self, physics: &mut PhysicsBox, time_step: f32) {
        physics.velocity.y = accelerate1d(physics.velocity.y, -self.config.fall_accel, time_step)
            .max(-self.config.max_speed_falling);
        // Air control
        physics.velocity.x = accelerate1d(
            physics.velocity.x,
            self.intent.x * self.config.max_speed_walking,
            time_step,
        )
        .min(self.config.max_speed_running)
        .max(-self.config.max_speed_running);
        if physics.velocity.y < 0.0 {
            self.animation_state = if self.stance == PlayerStance::Crouching {
                PlayerAnimationState::Crouching
            } else {
                PlayerAnimationState::Falling
            };
        }
    }

    pub fn ground_slide(&mut self, physics: &mut PhysicsBox, time_step: f32) {
        physics.velocity.x = decelerate1d(physics.velocity.x, self.config.decel_ground, time_step);
        self.animation_state = if self.stance == PlayerStance::Crouching {
            PlayerAnimationState::Crouching
        } else {
            if physics.velocity.x.abs() < MINIMUM_CLIP {
                PlayerAnimationState::Standing
            } else {
                PlayerAnimationState::Stopping
            }
        };
    }

    pub fn ground_move(&mut self, physics: &mut PhysicsBox, time_step: f32) {
        // If the player is trying to change directions, use the ground decel as
        // assistance.
        let (base_accel, max_speed) = if self.running {
            self.animation_state = if self.stance == PlayerStance::Crouching {
                PlayerAnimationState::Crouching
            } else {
                PlayerAnimationState::Running
            };
            (self.config.accel_running, self.config.max_speed_running)
        } else {
            self.animation_state = if self.stance == PlayerStance::Crouching {
                PlayerAnimationState::Crouching
            } else {
                PlayerAnimationState::Walking
            };
            (self.config.accel_walking, self.config.max_speed_walking)
        };
        let accel = if self.intent.x.signum() != physics.velocity.x.signum() {
            base_accel + self.config.decel_ground
        } else {
            base_accel
        };
        physics.velocity.x = accelerate1d(physics.velocity.x, self.intent.x * accel, time_step)
            .max(-max_speed)
            .min(max_speed);
    }

    pub fn run(&mut self) {
        if self.on_ground {
            self.running = true;
        }
    }

    pub fn reset_frame(&mut self) {
        self.intent = Vector2::new(0.0, 0.0);
        self.running = false;
        self.jump_edge = false;
    }

    pub fn climb(&mut self) {
        // Refuse to climb if holding jump.
        if self.jumping {
            return;
        }
        self.stance = PlayerStance::Climbing;
    }

    pub fn climb_move(&mut self, physics: &mut PhysicsBox, time_step: f32) {
        if self.intent.y == 0.0 && self.intent.x == 0.0 {
            physics.velocity.x = 0.0;
            physics.velocity.y = 0.0;
            return;
        }

        self.animation_state = PlayerAnimationState::Climbing;
        physics.velocity.x = accelerate1d(physics.velocity.x, self.intent.x * self.config.accel_climbing, time_step)
            .max(-self.config.max_speed_climbing)
            .min(self.config.max_speed_climbing);
        physics.velocity.y = accelerate1d(physics.velocity.y, self.intent.y * self.config.accel_climbing, time_step)
            .max(-self.config.max_speed_climbing)
            .min(self.config.max_speed_climbing);
    }

    pub fn initial_stance(&mut self) {
        let intends_crouch = (self.on_ground || self.stance == PlayerStance::Crouching) && self.intent.y < 0.0;
        if self.stance == PlayerStance::Climbing {
        } else if self.blocked || intends_crouch {
            self.stance = PlayerStance::Crouching;
            self.running = false;
        } else {
            self.stance = PlayerStance::Standing;
        }
    }

    pub fn update_bounding_box(&self, physics: &mut PhysicsBox) {
        physics.bbox = if self.stance == PlayerStance::Crouching {
            *CROUCHING_BBOX
        } else {
            *STANDING_BBOX
        }
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
                self.animation_state = PlayerAnimationState::Damaged;
                true
            }
            None => {
                self.animation_state = PlayerAnimationState::Dying;
                false
            }
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

pub fn initialize_player(
    world: &mut World,
    sprite_sheet: Handle<SpriteSheet>,
    player_start: Vector2<f32>,
) {
    let config = *world.read_resource::<PlayerConfig>();

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet,
        sprite_number: 0,
    };

    let physics_box = PhysicsBox::new(*STANDING_BBOX);
    let mut transform = Transform::default();
    transform.set_translation_xyz(player_start.x, player_start.y, 0.0);
    world
        .create_entity()
        .with(sprite_render)
        .with(Player::new(config))
        .with(physics_box)
        .with(transform)
        .build();
}

pub struct PlayerInputSystem;

impl<'s> System<'s> for PlayerInputSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut players, input): Self::SystemData) {
        for player in (&mut players).join() {
            player.reset_frame();

            if input.action_is_down("run").unwrap_or(false) {
                player.run();
            }
            if let Some(mv_x_axis) = input.axis_value("x") {
                player.intent.x = mv_x_axis;
            }
            if let Some(mv_y_axis) = input.axis_value("y") {
                player.intent.y = mv_y_axis;
            }
            let jump_down = input.action_is_down("jump").unwrap_or(false);
            player.jump_edge = jump_down && !player.jumping;
            player.jumping = jump_down;
            debug!("intent: {:?}", player.intent);
        }
    }
}

pub struct PlayerSpriteSystem;

impl<'s> System<'s> for PlayerSpriteSystem {
    type SystemData = (ReadStorage<'s, Player>, WriteStorage<'s, SpriteRender>);

    fn run(&mut self, (players, mut renders): Self::SystemData) {
        for (player, render) in (&players, &mut renders).join() {
            match player.animation_state {
                PlayerAnimationState::Crouching => {
                    render.sprite_number = 1;
                }
                _ => {
                    render.sprite_number = 0;
                }
            }
        }
    }
}

pub struct PlayerVelocitySystem;

impl<'s> System<'s> for PlayerVelocitySystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, PhysicsBox>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut players, mut physics_box, time): Self::SystemData) {
        let time_step = time.delta_seconds();
        for (player, physics) in (&mut players, &mut physics_box).join() {
            player.initial_stance();
            player.game_counter += 1;

            if player.on_ground && player.stance == PlayerStance::Climbing && player.intent.y < 0.0 {
                player.stance = PlayerStance::Standing;
                continue;
            }
            player.maybe_jump(physics);

            if player.stance == PlayerStance::Climbing {
                player.climb_move(physics, time_step);
                continue;
            }

            if player.on_ground {
                if player.intent.x == 0.0 {
                    player.ground_slide(physics, time_step);
                } else {
                    player.ground_move(physics, time_step);
                }
                player.update_bounding_box(physics);
            }  else {
                player.fall(physics, time_step);
                player.update_bounding_box(physics);
            }
            debug!("Move: {:?}\nPlayer: {:?}", physics, player);
        }
    }
}
