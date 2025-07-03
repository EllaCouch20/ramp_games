// Enemy system modules
pub mod fly_bullets;
pub mod fly_manager;
pub mod fly_movement;
pub mod fly_patterns;
pub mod fly_state;
pub mod fly_utils;

pub use fly_bullets::EnemyBullets;
pub use fly_manager::EnemyManager;
pub use fly_movement::EnemyMovement;
pub use fly_patterns::EnemyPatterns;
pub use fly_state::{EnemyState, EnemyGlobalState};

pub use fly_utils::{
    is_enemy,
    is_enemy_bullet,
    is_tiki,
    count_active_enemies,
};

pub const ENEMY_BULLET_SPEED: f32 = 6.0;
pub const PULSE_AMPLITUDE: f32 = 5.0;
pub const PULSE_SPEED: f32 = 0.1;