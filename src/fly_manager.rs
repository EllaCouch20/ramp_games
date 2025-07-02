use pelican_ui::Context;
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::fly_state::{EnemyState, EnemyGlobalState};
use crate::fly_patterns::EnemyPatterns;
use crate::fly_bullets::EnemyBullets;
use crate::fly_movement::EnemyMovement;

use crate::fly_utils;


static ENEMY_GLOBALS: OnceLock<Mutex<EnemyGlobalState>> = OnceLock::new();

pub struct EnemyManager;

impl EnemyManager {
    pub fn initialize() {
        ENEMY_GLOBALS.set(Mutex::new(EnemyGlobalState::default())).ok();
    }

    pub fn get_current_state() -> EnemyState {
        ENEMY_GLOBALS
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .enemy_state
            .clone()
    }

    pub fn get_wave_count() -> u32 {
        ENEMY_GLOBALS
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .wave_count
    }

    pub fn create_enemies(ctx: &mut Context, board: &mut Gameboard) {
        let (board_width, board_height) = board.0.size(ctx);

        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();

        globals.base_positions.clear();
        globals.enemy_last_shot_times.clear();

        let enemies = match globals.enemy_state {
            EnemyState::Initial => {
                globals.enemy_state = EnemyState::Pattern1;
                EnemyPatterns::get_initial_pattern(board_width, board_height)
            },
            EnemyState::Pattern1 => {
                globals.enemy_state = EnemyState::Pattern2;
                globals.wave_count += 1;
                EnemyPatterns::get_pattern_1(board_width, board_height)
            },
            EnemyState::Pattern2 => {
                globals.enemy_state = EnemyState::Pattern3;
                globals.wave_count += 1;
                EnemyPatterns::get_pattern_2(board_width, board_height)
            },
            EnemyState::Pattern3 => {
                globals.enemy_state = EnemyState::Pattern4;
                globals.wave_count += 1;
                EnemyPatterns::get_pattern_3(board_width, board_height)
            },
            EnemyState::Pattern4 => {
                globals.enemy_state = EnemyState::Pattern1;
                globals.wave_count += 1;
                EnemyPatterns::get_pattern_4(board_width, board_height)
            },
            EnemyState::AllDestroyed => {
                globals.wave_count += 1;

                match globals.wave_count % 4 {
                    1 => {
                        globals.enemy_state = EnemyState::Pattern1;
                        EnemyPatterns::get_pattern_1(board_width, board_height)
                    },
                    2 => {
                        globals.enemy_state = EnemyState::Pattern2;
                        EnemyPatterns::get_pattern_2(board_width, board_height)
                    },
                    3 => {
                        globals.enemy_state = EnemyState::Pattern3;
                        EnemyPatterns::get_pattern_3(board_width, board_height)
                    },
                    0 => {
                        globals.enemy_state = EnemyState::Pattern4;
                        EnemyPatterns::get_pattern_4(board_width, board_height)
                    },
                    _ => {
                        globals.enemy_state = EnemyState::Pattern1;
                        EnemyPatterns::get_pattern_1(board_width, board_height)
                    }
                }
            },
        };

        for (id, image, x, y) in enemies {
            let sprite = Sprite::new(ctx, id, image, (50.0, 50.0), (Offset::Static(x), Offset::Static(y)));
            board.insert_sprite(ctx, sprite);

            globals.base_positions.insert(id.to_string(), (x, y));

            if fly_utils::is_enemy(id) {
                globals.enemy_last_shot_times.insert(id.to_string(), Instant::now());
            }
        }

        println!("Created {} enemies for wave {}", globals.base_positions.len(), globals.wave_count);
    }

    pub fn check_and_manage_enemy_state(ctx: &mut Context, board: &mut Gameboard) {
        let enemy_count = fly_utils::count_active_enemies(board);

        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
        match globals.enemy_state {
            EnemyState::AllDestroyed => {
                return;
            },
            _ => {
                if enemy_count == 0 {
                    println!("*** ALL ENEMIES DESTROYED! Preparing next wave... ***");
                    globals.enemy_state = EnemyState::AllDestroyed;

                    drop(globals);

                    EnemyBullets::clear_all_enemy_bullets(ctx, board);
                    Self::create_enemies(ctx, board);
                }
            }
        }
    }

pub fn update_enemy_pulse(ctx: &mut Context, board: &mut Gameboard) {
    let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
    let base_positions = globals.base_positions.clone(); 
    EnemyMovement::update_enemy_pulse(ctx, board, &mut globals.pulse_time, &base_positions);
}

    pub fn update_enemy_shooting(ctx: &mut Context, board: &mut Gameboard) {
        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
        EnemyBullets::update_enemy_shooting(ctx, board, &mut globals.enemy_last_shot_times);
    }

    pub fn update_enemy_bullets(ctx: &mut Context, board: &mut Gameboard) -> Vec<String> {
        EnemyBullets::update_enemy_bullets(ctx, board)
    }

    pub fn get_active_enemy_bullets(board: &mut Gameboard, ctx: &mut Context) -> Vec<(String, (f32, f32), (f32, f32))> {
        EnemyBullets::get_active_enemy_bullets(board, ctx)
    }

    pub fn remove_enemy_from_base_positions(enemy_id: &str) {
        if let Some(globals) = ENEMY_GLOBALS.get() {
            let mut globals = globals.lock().unwrap();
            globals.base_positions.remove(enemy_id);
            globals.enemy_last_shot_times.remove(enemy_id);
        }
    }

    pub fn is_enemy(sprite_id: &str) -> bool {
        fly_utils::is_enemy(sprite_id)
    }

    pub fn is_enemy_bullet(sprite_id: &str) -> bool {
        fly_utils::is_enemy_bullet(sprite_id)
    }

    pub fn is_tiki(sprite_id: &str) -> bool {
        fly_utils::is_tiki(sprite_id)
    }
}