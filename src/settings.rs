use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub can_fly: bool,
    pub can_shoot: bool,

    pub player_auto_shoot: bool,
    pub player_auto_move: bool,

    pub player_invincible: bool,

    pub bullet_speed_fly: f32,
    pub bullet_speed_player: f32,

    pub peak_min: f32,
}

impl GameSettings {
    pub fn new() -> Self {
        Self {
            can_fly: false,
            can_shoot: true,
            player_auto_shoot: false,
            player_auto_move: false,
            player_invincible: false,
            bullet_speed_fly: 800.0,
            bullet_speed_player: 600.0,
            peak_min: 500.0,
        }
    }

    pub fn toggle_can_fly(&mut self) {
        self.can_fly = !self.can_fly;
    }

    pub fn toggle_can_shoot(&mut self) {
        self.can_shoot = !self.can_shoot;
    }

    pub fn toggle_auto_shoot(&mut self) {
        self.player_auto_shoot = !self.player_auto_shoot;
    }

    pub fn toggle_auto_move(&mut self) {
        self.player_auto_move = !self.player_auto_move;
    }

    pub fn toggle_invincible(&mut self) {
        self.player_invincible = !self.player_invincible;
    }

    pub fn set_bullet_speed_fly(&mut self, speed: f32) {
        self.bullet_speed_fly = speed;
    }

    pub fn set_bullet_speed_player(&mut self, speed: f32) {
        self.bullet_speed_player = speed;
    }

    pub fn set_peak_min(&mut self, peak: f32) {
        self.peak_min = peak;
    }

    pub fn get_peak_min(&self) -> f32 {
        self.peak_min
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::new()
    }
}
