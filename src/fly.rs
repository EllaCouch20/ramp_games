use pelican_ui::{Context};
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use rand::Rng;

const PULSE_AMPLITUDE: f32 = 5.0; 
const PULSE_SPEED: f32 = 0.1;
const ENEMY_BULLET_SPEED: f32 = 6.0;
const TIKI_SHOOT_COOLDOWN: Duration = Duration::from_millis(2000);
const TIKI_SHOOT_CHANCE: f32 = 0.3; 

static mut ENEMY_BASE_POSITIONS: Option<HashMap<String, (f32, f32)>> = None;
static mut PULSE_TIME: f32 = 0.0;
static mut TIKI_LAST_SHOT_TIMES: Option<HashMap<String, Instant>> = None;

pub struct EnemyManager;

impl EnemyManager {
    pub fn initialize() {
        unsafe {
            ENEMY_BASE_POSITIONS = Some(HashMap::new());
            TIKI_LAST_SHOT_TIMES = Some(HashMap::new());
            PULSE_TIME = 0.0;
        }
    }

    pub fn create_enemies(ctx: &mut Context, board: &mut Gameboard) {
        let (board_width, board_height) = board.0.size(ctx);
        let enemies = vec![
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
        ];

        unsafe {
            if let Some(ref mut base_positions) = ENEMY_BASE_POSITIONS {
                if let Some(ref mut shot_times) = TIKI_LAST_SHOT_TIMES {
                    for (id, image, x, y) in enemies {
                        let sprite = Sprite::new(ctx, id, image, (50.0, 50.0), (Offset::Static(x), Offset::Static(y)));
                        board.insert_sprite(ctx, sprite);
                        
                        base_positions.insert(id.to_string(), (x, y));
                        
                        // Initialize shot times for tiki enemies
                        if id.starts_with("tiki_") {
                            shot_times.insert(id.to_string(), Instant::now());
                        }
                    }
                }
            }
        }
    }

    pub fn update_enemy_pulse(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            PULSE_TIME += PULSE_SPEED;
            
            if let Some(ref base_positions) = ENEMY_BASE_POSITIONS {
                let pulse_scale = (PULSE_TIME.sin() + 1.0) * 0.5;
                let pulse_offset = pulse_scale * PULSE_AMPLITUDE;
                
                let center_x = base_positions.values().map(|(x, _)| *x).sum::<f32>() / base_positions.len() as f32;
                let center_y = base_positions.values().map(|(_, y)| *y).sum::<f32>() / base_positions.len() as f32;
                
                for (enemy_id, &(base_x, base_y)) in base_positions.iter() {
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
        }
    }

    pub fn update_tiki_shooting(ctx: &mut Context, board: &mut Gameboard) {
        let now = Instant::now();
        let mut tikis_to_shoot = Vec::new();
        
        unsafe {
            if let Some(ref mut shot_times) = TIKI_LAST_SHOT_TIMES {
                for (tiki_id, last_shot) in shot_times.iter_mut() {
                    if now.duration_since(*last_shot) >= TIKI_SHOOT_COOLDOWN {
                        let mut rng = rand::thread_rng();
                        if rng.gen_range(0.0..1.0) < TIKI_SHOOT_CHANCE {
                            if let Some(sprite) = board.2.get_mut(tiki_id) {
                                let pos = sprite.position(ctx);
                                let size = *sprite.dimensions();
                                tikis_to_shoot.push((tiki_id.clone(), pos, size));
                                *last_shot = now;
                            }
                        } else {
                            *last_shot = now;
                        }
                    }
                }
            }
        }
        
        for (tiki_id, pos, size) in tikis_to_shoot {
            Self::tiki_shoot(ctx, board, pos, size);
        }
    }

    fn tiki_shoot(ctx: &mut Context, board: &mut Gameboard, tiki_pos: (f32, f32), tiki_size: (f32, f32)) {
        let bullet_size = (12.0, 12.0);
        let (x, y) = tiki_pos;
        let bullet_id = format!("enemy_bullet_{}", uuid::Uuid::new_v4());
        
        let bullet = Sprite::new(
            ctx,
            &bullet_id,
            "bullet_downward.png",
            bullet_size,
            (
                Offset::Static(x + ((tiki_size.0 - bullet_size.0) / 2.0)), 
                Offset::Static(y + tiki_size.1), 
            ),
        );
        board.insert_sprite(ctx, bullet);
    }

    pub fn update_enemy_bullets(ctx: &mut Context, board: &mut Gameboard) -> Vec<String> {
        let mut bullets_to_remove = Vec::new();
        let (_, board_height) = board.0.size(ctx);

        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("enemy_bullet_") {
                // Move bullet downward
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
        unsafe {
            if let Some(ref mut base_positions) = ENEMY_BASE_POSITIONS {
                base_positions.remove(enemy_id);
            }
            if enemy_id.starts_with("tiki_") {
                if let Some(ref mut shot_times) = TIKI_LAST_SHOT_TIMES {
                    shot_times.remove(enemy_id);
                }
            }
        }
    }

    pub fn is_enemy(sprite_id: &str) -> bool {
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