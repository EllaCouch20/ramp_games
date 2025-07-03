pub mod manager;
pub mod movement;
pub mod lives;

pub use manager::PlayerManager;
pub use lives::{PlayerLives, LivesDisplayInfo};

use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub enum PlayerState {
    Idle { last_shot: Option<Instant> },
    MovingLeft { last_shot: Option<Instant>, speed: f32 },
    MovingRight { last_shot: Option<Instant>, speed: f32 },
    MovingBoth { last_shot: Option<Instant>, left_speed: f32, right_speed: f32 },
    Shooting { direction: MovementDirection, shot_time: Instant },
    Destroyed,
}

#[derive(Clone, Copy, Debug)]
pub enum MovementDirection {
    None,
    Left,
    Right,
    Both,
}

#[derive(Clone, Copy)]
pub struct KeysHeld {
    pub left: bool,
    pub right: bool,
}

impl KeysHeld {
    pub const fn new() -> Self {
        Self { left: false, right: false }
    }

    pub fn to_direction(&self) -> MovementDirection {
        match (self.left, self.right) {
            (true, true) => MovementDirection::Both,
            (true, false) => MovementDirection::Left,
            (false, true) => MovementDirection::Right,
            (false, false) => MovementDirection::None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ServerMovement {
    pub direction: MovementDirection,
    pub start_time: Instant,
}

// Constants
pub const STEP: f32 = 1.5;
pub const BULLET_SPEED: f32 = 8.0;
pub const MOVEMENT_SPEED: f32 = 300.0;
pub const SHOOT_COOLDOWN: std::time::Duration = std::time::Duration::from_millis(200);
pub const SERVER_MOVEMENT_DURATION: std::time::Duration = std::time::Duration::from_millis(100);