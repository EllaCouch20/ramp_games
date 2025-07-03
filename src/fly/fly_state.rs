use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyState {
    Initial,
    Pattern1,
    Pattern2,
    Pattern3,
    AllDestroyed,
}

pub struct EnemyGlobalState {
    pub base_positions: HashMap<String, (f32, f32)>,
    pub pulse_time: f32,
    pub enemy_last_shot_times: HashMap<String, Instant>,
    pub enemy_state: EnemyState,
    pub wave_count: u32,
}

impl Default for EnemyGlobalState {
    fn default() -> Self {
        Self {
            base_positions: HashMap::new(),
            pulse_time: 0.0,
            enemy_last_shot_times: HashMap::new(),
            enemy_state: EnemyState::Initial,
            wave_count: 0,
        }
    }
}