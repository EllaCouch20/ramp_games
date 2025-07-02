use pelican_ui::{Context};
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Mutex, OnceLock};
use rand::Rng;

const PULSE_AMPLITUDE: f32 = 5.0;
const PULSE_SPEED: f32 = 0.1;
const ENEMY_BULLET_SPEED: f32 = 6.0;
const ENEMY_SHOOT_COOLDOWN: Duration = Duration::from_millis(1500); // All enemies can shoot
const ENEMY_SHOOT_CHANCE: f32 = 0.3; // Shooting chance for all enemies

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyState {
    Initial,
    Pattern1,
    Pattern2,
    Pattern3,
    Pattern4,
    AllDestroyed,
}

struct EnemyGlobalState {
    base_positions: HashMap<String, (f32, f32)>,
    pulse_time: f32,
    enemy_last_shot_times: HashMap<String, Instant>, // Track all enemies, not just tikis
    enemy_state: EnemyState,
    wave_count: u32,
}

impl Default for EnemyGlobalState {
    fn default() -> Self {
        Self {
            base_positions: HashMap::new(),
            pulse_time: 0.0,
            enemy_last_shot_times: HashMap::new(), // Track all enemies
            enemy_state: EnemyState::Initial,
            wave_count: 0,
        }
    }
}

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

        // Clear old tracking data
        globals.base_positions.clear();
        globals.enemy_last_shot_times.clear();

        let enemies = match globals.enemy_state {
            EnemyState::Initial => {
                println!("*** SPAWNING INITIAL ENEMY WAVE ***");
                globals.enemy_state = EnemyState::Pattern1;
                Self::get_initial_pattern(board_width, board_height)
            },
            EnemyState::Pattern1 => {
                println!("*** SPAWNING PATTERN 1 ENEMIES ***");
                globals.enemy_state = EnemyState::Pattern2;
                globals.wave_count += 1;
                Self::get_pattern_1(board_width, board_height)
            },
            EnemyState::Pattern2 => {
                println!("*** SPAWNING PATTERN 2 ENEMIES ***");
                globals.enemy_state = EnemyState::Pattern3;
                globals.wave_count += 1;
                Self::get_pattern_2(board_width, board_height)
            },
            EnemyState::Pattern3 => {
                println!("*** SPAWNING PATTERN 3 ENEMIES ***");
                globals.enemy_state = EnemyState::Pattern4;
                globals.wave_count += 1;
                Self::get_pattern_3(board_width, board_height)
            },
            EnemyState::Pattern4 => {
                println!("*** SPAWNING PATTERN 4 ENEMIES ***");
                globals.enemy_state = EnemyState::Pattern1; // Loop back to Pattern1
                globals.wave_count += 1;
                Self::get_pattern_4(board_width, board_height)
            },
            EnemyState::AllDestroyed => {
                println!("*** ALL ENEMIES DESTROYED - RESPAWNING ***");
                // Don't change the state here, let it continue with the current pattern cycle
                globals.wave_count += 1;
                
                // Continue with the next pattern in sequence
                match globals.wave_count % 4 {
                    1 => {
                        globals.enemy_state = EnemyState::Pattern1;
                        Self::get_pattern_1(board_width, board_height)
                    },
                    2 => {
                        globals.enemy_state = EnemyState::Pattern2;
                        Self::get_pattern_2(board_width, board_height)
                    },
                    3 => {
                        globals.enemy_state = EnemyState::Pattern3;
                        Self::get_pattern_3(board_width, board_height)
                    },
                    0 => {
                        globals.enemy_state = EnemyState::Pattern4;
                        Self::get_pattern_4(board_width, board_height)
                    },
                    _ => {
                        globals.enemy_state = EnemyState::Pattern1;
                        Self::get_pattern_1(board_width, board_height)
                    }
                }
            },
        };

        for (id, image, x, y) in enemies {
            let sprite = Sprite::new(ctx, id, image, (50.0, 50.0), (Offset::Static(x), Offset::Static(y)));
            board.insert_sprite(ctx, sprite);

            globals.base_positions.insert(id.to_string(), (x, y));

            // Track ALL enemies for shooting, not just tikis
            if Self::is_enemy(id) {
                globals.enemy_last_shot_times.insert(id.to_string(), Instant::now());
            }
        }

        println!("Created {} enemies for wave {}", globals.base_positions.len(), globals.wave_count);
    }

    fn get_initial_pattern(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        vec![
            ("b2_1", "b-2.png", board_width * 0.2, board_height * 0.1),
            ("b2_2", "b-2.png", board_width * 0.4, board_height * 0.1),
            ("b2_3", "b-2.png", board_width * 0.6, board_height * 0.1),
            ("b2_4", "b-2.png", board_width * 0.8, board_height * 0.1),
            ("tiki_1", "tiki_fly.png", board_width * 0.15, board_height * 0.2),
            ("tiki_2", "tiki_fly.png", board_width * 0.3, board_height * 0.2),
            ("tiki_3", "tiki_fly.png", board_width * 0.5, board_height * 0.2),
            ("tiki_4", "tiki_fly.png", board_width * 0.7, board_height * 0.2),
            ("tiki_5", "tiki_fly.png", board_width * 0.85, board_height * 0.2),
            ("northrop_1", "northrop.png", board_width * 0.25, board_height * 0.3),
            ("northrop_2", "northrop.png", board_width * 0.4, board_height * 0.3),
            ("northrop_3", "northrop.png", board_width * 0.6, board_height * 0.3),
            ("northrop_4", "northrop.png", board_width * 0.75, board_height * 0.3),
        ]
    }

    fn get_pattern_1(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // Diamond formation
        vec![
            ("b2_1", "b-2.png", board_width * 0.5, board_height * 0.05),
            ("b2_2", "b-2.png", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b-2.png", board_width * 0.7, board_height * 0.15),
            ("tiki_1", "tiki_fly.png", board_width * 0.1, board_height * 0.25),
            ("tiki_2", "tiki_fly.png", board_width * 0.5, board_height * 0.25),
            ("tiki_3", "tiki_fly.png", board_width * 0.9, board_height * 0.25),
            ("northrop_1", "northrop.png", board_width * 0.2, board_height * 0.35),
            ("northrop_2", "northrop.png", board_width * 0.4, board_height * 0.35),
            ("northrop_3", "northrop.png", board_width * 0.6, board_height * 0.35),
            ("northrop_4", "northrop.png", board_width * 0.8, board_height * 0.35),
        ]
    }

    fn get_pattern_2(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // V formation
        vec![
            ("b2_1", "b-2.png", board_width * 0.1, board_height * 0.1),
            ("b2_2", "b-2.png", board_width * 0.3, board_height * 0.15),
            ("b2_3", "b-2.png", board_width * 0.5, board_height * 0.2),
            ("b2_4", "b-2.png", board_width * 0.7, board_height * 0.15),
            ("b2_5", "b-2.png", board_width * 0.9, board_height * 0.1),
            ("tiki_1", "tiki_fly.png", board_width * 0.2, board_height * 0.3),
            ("tiki_2", "tiki_fly.png", board_width * 0.4, board_height * 0.25),
            ("tiki_3", "tiki_fly.png", board_width * 0.6, board_height * 0.25),
            ("tiki_4", "tiki_fly.png", board_width * 0.8, board_height * 0.3),
            ("northrop_1", "northrop.png", board_width * 0.35, board_height * 0.4),
            ("northrop_2", "northrop.png", board_width * 0.65, board_height * 0.4),
        ]
    }

    fn get_pattern_3(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // Circular formation
        let center_x = board_width * 0.5;
        let center_y = board_height * 0.25;
        let radius = board_width * 0.2;

        vec![
            ("b2_1", "b-2.png", center_x + radius * 0.0, center_y - radius),
            ("b2_2", "b-2.png", center_x + radius * 0.707, center_y - radius * 0.707),
            ("b2_3", "b-2.png", center_x + radius, center_y),
            ("b2_4", "b-2.png", center_x + radius * 0.707, center_y + radius * 0.707),
            ("b2_5", "b-2.png", center_x, center_y + radius),
            ("b2_6", "b-2.png", center_x - radius * 0.707, center_y + radius * 0.707),
            ("b2_7", "b-2.png", center_x - radius, center_y),
            ("b2_8", "b-2.png", center_x - radius * 0.707, center_y - radius * 0.707),
            ("tiki_1", "tiki_fly.png", center_x, center_y),
            ("tiki_2", "tiki_fly.png", center_x + radius * 0.5, center_y),
            ("tiki_3", "tiki_fly.png", center_x - radius * 0.5, center_y),
            ("northrop_1", "northrop.png", board_width * 0.1, board_height * 0.4),
            ("northrop_2", "northrop.png", board_width * 0.9, board_height * 0.4),
        ]
    }

    fn get_pattern_4(board_width: f32, board_height: f32) -> Vec<(&'static str, &'static str, f32, f32)> {
        // Random chaos formation
        let mut enemies = Vec::new();
        let mut rng = rand::thread_rng();

        // Create b2 enemies
        for i in 1..=6 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.05..0.2) * board_height;
            let id: &'static str = Box::leak(format!("b2_{}", i).into_boxed_str());
            enemies.push((id, "b-2.png", x, y));
        }

        // Create tiki enemies
        for i in 1..=8 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.2..0.35) * board_height;
            let id: &'static str = Box::leak(format!("tiki_{}", i).into_boxed_str());
            enemies.push((id, "tiki_fly.png", x, y));
        }

        // Create northrop enemies
        for i in 1..=5 {
            let x = rng.gen_range(0.1..0.9) * board_width;
            let y = rng.gen_range(0.35..0.45) * board_height;
            let id: &'static str = Box::leak(format!("northrop_{}", i).into_boxed_str());
            enemies.push((id, "northrop.png", x, y));
        }

        enemies
    }

    pub fn check_and_manage_enemy_state(ctx: &mut Context, board: &mut Gameboard) {
        let enemy_count = Self::count_active_enemies(board);

        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
        match globals.enemy_state {
            EnemyState::AllDestroyed => {
                // Already handled in create_enemies
                return;
            },
            _ => {
                if enemy_count == 0 {
                    println!("*** ALL ENEMIES DESTROYED! Preparing next wave... ***");
                    globals.enemy_state = EnemyState::AllDestroyed;

                    drop(globals);

                    Self::clear_all_enemy_bullets(ctx, board);
                    Self::create_enemies(ctx, board);
                }
            }
        }
    }

    fn count_active_enemies(board: &Gameboard) -> usize {
        let count = board.2.keys().filter(|id| Self::is_enemy(id)).count();
        println!("Active enemies: {}", count);
        count
    }

    fn clear_all_enemy_bullets(ctx: &mut Context, board: &mut Gameboard) {
        let bullet_ids: Vec<String> = board.2.keys()
            .filter(|id| id.starts_with("enemy_bullet_"))
            .cloned()
            .collect();

        for bullet_id in bullet_ids {
            if board.2.remove(&bullet_id).is_some() {
                board.0.0.remove(&bullet_id);
            }
        }
    }

    pub fn update_enemy_pulse(ctx: &mut Context, board: &mut Gameboard) {
        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
        globals.pulse_time += PULSE_SPEED;

        let pulse_scale = (globals.pulse_time.sin() + 1.0) * 0.5;
        let pulse_offset = pulse_scale * PULSE_AMPLITUDE;

        // Calculate center of all enemies
        if globals.base_positions.is_empty() {
            return;
        }

        let center_x = globals.base_positions.values().map(|(x, _)| *x).sum::<f32>() / globals.base_positions.len() as f32;
        let center_y = globals.base_positions.values().map(|(_, y)| *y).sum::<f32>() / globals.base_positions.len() as f32;

        for (enemy_id, &(base_x, base_y)) in globals.base_positions.iter() {
            if let Some(sprite) = board.2.get_mut(enemy_id) {
                let dx = base_x - center_x;
                let dy = base_y - center_y;

                let distance = (dx * dx + dy * dy).sqrt();
                let (norm_dx, norm_dy) = if distance > 0.0 {
                    (dx / distance, dy / distance)
                } else {
                    (0.0, 0.0)
                };

                let pulse_x = norm_dx * pulse_offset;
                let pulse_y = norm_dy * pulse_offset;

                sprite.adjustments().0 = pulse_x;
                sprite.adjustments().1 = pulse_y;
            }
        }
    }

    pub fn update_enemy_shooting(ctx: &mut Context, board: &mut Gameboard) {
        let now = Instant::now();
        let mut enemies_to_shoot = Vec::new();

        let mut globals = ENEMY_GLOBALS.get().unwrap().lock().unwrap();
        
        // Get all active enemies and ensure they're being tracked
        let active_enemies: Vec<String> = board.2.keys()
            .filter(|id| Self::is_enemy(id))
            .cloned()
            .collect();

        // Add any missing enemies to the shot tracking
        for enemy_id in &active_enemies {
            if !globals.enemy_last_shot_times.contains_key(enemy_id) {
                globals.enemy_last_shot_times.insert(enemy_id.clone(), Instant::now());
            }
        }

        // Remove tracking for destroyed enemies
        let tracked_enemies: Vec<String> = globals.enemy_last_shot_times.keys().cloned().collect();
        for enemy_id in tracked_enemies {
            if !active_enemies.contains(&enemy_id) {
                globals.enemy_last_shot_times.remove(&enemy_id);
            }
        }

        // Check shooting for all tracked enemies
        for (enemy_id, last_shot) in globals.enemy_last_shot_times.iter_mut() {
            if now.duration_since(*last_shot) >= ENEMY_SHOOT_COOLDOWN {
                let mut rng = rand::thread_rng();
                if rng.gen_range(0.0..1.0) < ENEMY_SHOOT_CHANCE {
                    if let Some(sprite) = board.2.get_mut(enemy_id) {
                        let pos = sprite.position(ctx);
                        let size = *sprite.dimensions();
                        enemies_to_shoot.push((enemy_id.clone(), pos, size));
                        *last_shot = now;
                    }
                } else {
                    // Reset cooldown even if not shooting to prevent immediate retry
                    *last_shot = now;
                }
            }
        }
        drop(globals);

        for (enemy_id, pos, size) in enemies_to_shoot {
            Self::enemy_shoot(ctx, board, pos, size);
        }
    }

    fn enemy_shoot(ctx: &mut Context, board: &mut Gameboard, enemy_pos: (f32, f32), enemy_size: (f32, f32)) {
        let bullet_size = (12.0, 12.0);
        let (x, y) = enemy_pos;
        let bullet_id = format!("enemy_bullet_{}", uuid::Uuid::new_v4());

        let bullet = Sprite::new(
            ctx,
            &bullet_id,
            "bullet_downward.png",
            bullet_size,
            (
                Offset::Static(x + ((enemy_size.0 - bullet_size.0) / 2.0)),
                Offset::Static(y + enemy_size.1),
            ),
        );
        board.insert_sprite(ctx, bullet);
    }

    pub fn update_enemy_bullets(ctx: &mut Context, board: &mut Gameboard) -> Vec<String> {
        let mut bullets_to_remove = Vec::new();
        let (_, board_height) = board.0.size(ctx);

        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("enemy_bullet_") {
                sprite.adjustments().1 += ENEMY_BULLET_SPEED;
                let pos = sprite.position(ctx);

                if pos.1 > board_height + 50.0 {
                    bullets_to_remove.push(id.clone());
                }
            }
        }

        bullets_to_remove
    }

    pub fn get_active_enemy_bullets(board: &mut Gameboard, ctx: &mut Context) -> Vec<(String, (f32, f32), (f32, f32))> {
        let mut active_bullets = Vec::new();

        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("enemy_bullet_") {
                let pos = sprite.position(ctx);
                let size = *sprite.dimensions();
                active_bullets.push((id.clone(), pos, size));
            }
        }

        active_bullets
    }

    pub fn remove_enemy_from_base_positions(enemy_id: &str) {
        if let Some(globals) = ENEMY_GLOBALS.get() {
            let mut globals = globals.lock().unwrap();
            globals.base_positions.remove(enemy_id);

            // Remove from shooting tracking for any enemy type
            globals.enemy_last_shot_times.remove(enemy_id);
        }
    }

    pub fn is_enemy(sprite_id: &str) -> bool {
        (sprite_id.starts_with("b2_") || 
         sprite_id.starts_with("tiki_") || 
         sprite_id.starts_with("northrop_")) &&
        !sprite_id.starts_with("player") &&
        !sprite_id.starts_with("bullet_") &&
        !sprite_id.starts_with("enemy_bullet_") &&
        !sprite_id.starts_with("explosion_")
    }

    pub fn is_enemy_bullet(sprite_id: &str) -> bool {
        sprite_id.starts_with("enemy_bullet_")
    }

    pub fn is_tiki(sprite_id: &str) -> bool {
        sprite_id.starts_with("tiki_")
    }
}