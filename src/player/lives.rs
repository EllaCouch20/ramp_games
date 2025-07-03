use pelican_ui::Context;
use pelican_game_engine::Gameboard;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::{PlayerState, PlayerManager};

const RESPAWN_DELAY: Duration = Duration::from_millis(2000);
const INVULNERABILITY_DURATION: Duration = Duration::from_millis(3000);
const DEFAULT_LIVES: u32 = 3;

struct PlayerLivesState {
    lives: u32,
    death_time: Option<Instant>,
    invulnerable_until: Option<Instant>,
}

static PLAYER_LIVES_STATE: Mutex<PlayerLivesState> = Mutex::new(PlayerLivesState {
    lives: DEFAULT_LIVES,
    death_time: None,
    invulnerable_until: None,
});

pub struct PlayerLives;

impl PlayerLives {
    pub fn initialize() {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.lives = DEFAULT_LIVES;
        state.death_time = None;
        state.invulnerable_until = None;
    }

    pub fn initialize_with_lives(lives: u32) {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.lives = lives;
        state.death_time = None;
        state.invulnerable_until = None;
    }

    pub fn get_lives() -> u32 {
        PLAYER_LIVES_STATE.lock().unwrap().lives
    }

    pub fn add_lives(amount: u32) {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.lives += amount;
    }

    pub fn is_invulnerable() -> bool {
        let state = PLAYER_LIVES_STATE.lock().unwrap();
        if let Some(invuln_time) = state.invulnerable_until {
            Instant::now() < invuln_time
        } else {
            false
        }
    }

    pub fn is_waiting_to_respawn() -> bool {
        let state = PLAYER_LIVES_STATE.lock().unwrap();
        let death_time_exists = state.death_time.is_some();
        death_time_exists && PlayerManager::is_player_destroyed()
    }

    pub fn is_game_over() -> bool {
        let state = PLAYER_LIVES_STATE.lock().unwrap();
        state.lives == 0 && PlayerManager::is_player_destroyed()
    }

    pub fn handle_player_death() {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        if state.lives > 0 {
            state.lives -= 1;
            state.death_time = Some(Instant::now());
            let remaining_lives = state.lives;
            drop(state);
            
            PlayerManager::destroy_player();
            println!("💀 Player died! Lives remaining: {}", remaining_lives);
        }
    }

    pub fn update(ctx: &mut Context, board: &mut Gameboard) {
        let should_respawn = {
            let mut state = PLAYER_LIVES_STATE.lock().unwrap();
            
            let mut should_respawn = false;
            if let Some(death_time) = state.death_time {
                if Instant::now().duration_since(death_time) >= RESPAWN_DELAY {
                    let current_lives = state.lives;
                    if current_lives > 0 {
                        should_respawn = true;
                    }
                    state.death_time = None;
                }
            }

            if let Some(invuln_time) = state.invulnerable_until {
                if Instant::now() >= invuln_time {
                    state.invulnerable_until = None;
                }
            }
            
            should_respawn
        };

        if should_respawn {
            Self::respawn_player(ctx, board);
        }
    }

    fn respawn_player(ctx: &mut Context, board: &mut Gameboard) {
        if board.2.contains_key("player") {
            board.2.remove("player");
        }

        let player = PlayerManager::create_player(ctx);
        board.insert_sprite(ctx, player);

        PlayerManager::initialize();

        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.invulnerable_until = Some(Instant::now() + INVULNERABILITY_DURATION);
        let remaining_lives = state.lives;
        println!("✨ Player respawned with {} lives remaining!", remaining_lives);
    }

    pub fn force_respawn(ctx: &mut Context, board: &mut Gameboard) {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.death_time = None;
        if state.lives == 0 {
            state.lives = 1;
        }
        drop(state);
        Self::respawn_player(ctx, board);
    }

    pub fn reset_for_new_game() {
        let mut state = PLAYER_LIVES_STATE.lock().unwrap();
        state.lives = DEFAULT_LIVES;
        state.death_time = None;
        state.invulnerable_until = None;
    }

    pub fn get_respawn_time_remaining() -> Option<f32> {
        let state = PLAYER_LIVES_STATE.lock().unwrap();
        if let Some(death_time) = state.death_time {
            let elapsed = Instant::now().duration_since(death_time);
            if elapsed < RESPAWN_DELAY {
                let remaining = RESPAWN_DELAY - elapsed;
                Some(remaining.as_secs_f32())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_invulnerability_time_remaining() -> Option<f32> {
        let state = PLAYER_LIVES_STATE.lock().unwrap();
        if let Some(invuln_time) = state.invulnerable_until {
            let now = Instant::now();
            if now < invuln_time {
                let remaining = invuln_time.duration_since(now);
                Some(remaining.as_secs_f32())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn can_take_damage() -> bool {
        !Self::is_invulnerable() && !PlayerManager::is_player_destroyed()
    }

    pub fn handle_enemy_collision() -> bool {
        if Self::can_take_damage() {
            Self::handle_player_death();
            true
        } else {
            false
        }
    }

    pub fn get_display_info() -> LivesDisplayInfo {
        LivesDisplayInfo {
            lives: Self::get_lives(),
            is_invulnerable: Self::is_invulnerable(),
            is_waiting_to_respawn: Self::is_waiting_to_respawn(),
            respawn_time_remaining: Self::get_respawn_time_remaining(),
            invulnerability_time_remaining: Self::get_invulnerability_time_remaining(),
            is_game_over: Self::is_game_over(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LivesDisplayInfo {
    pub lives: u32,
    pub is_invulnerable: bool,
    pub is_waiting_to_respawn: bool,
    pub respawn_time_remaining: Option<f32>,
    pub invulnerability_time_remaining: Option<f32>,
    pub is_game_over: bool,
}

impl LivesDisplayInfo {
    pub fn new(lives: u32) -> Self {
        Self {
            lives,
            is_invulnerable: false,
            is_waiting_to_respawn: false,
            respawn_time_remaining: None,
            invulnerability_time_remaining: None,
            is_game_over: lives == 0,
        }
    }
}