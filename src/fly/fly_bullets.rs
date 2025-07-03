use pelican_ui::Context;
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard};
use std::time::{Duration, Instant};
use rand::Rng;

const ENEMY_BULLET_SPEED: f32 = 6.0;
const ENEMY_SHOOT_COOLDOWN: Duration = Duration::from_millis(2500); 
const ENEMY_SHOOT_CHANCE: f32 = 0.1; 

pub struct EnemyBullets;

impl EnemyBullets {
    pub fn update_enemy_shooting(
        ctx: &mut Context, 
        board: &mut Gameboard,
        enemy_last_shot_times: &mut std::collections::HashMap<String, Instant>
    ) {
        let now = Instant::now();
        let mut enemies_to_shoot = Vec::new();

        let active_enemies: Vec<String> = board.2.keys()
            .filter(|id| crate::fly::fly_utils::is_enemy(id))
            .cloned()
            .collect();

        for enemy_id in &active_enemies {
            if !enemy_last_shot_times.contains_key(enemy_id) {
                enemy_last_shot_times.insert(enemy_id.clone(), Instant::now());
            }
        }

        let tracked_enemies: Vec<String> = enemy_last_shot_times.keys().cloned().collect();
        for enemy_id in tracked_enemies {
            if !active_enemies.contains(&enemy_id) {
                enemy_last_shot_times.remove(&enemy_id);
            }
        }

        for (enemy_id, last_shot) in enemy_last_shot_times.iter_mut() {
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
                    *last_shot = now;
                }
            }
        }

        for (_enemy_id, pos, size) in enemies_to_shoot {
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

    pub fn get_active_enemy_bullets(
        board: &mut Gameboard,
        ctx: &mut Context
    ) -> Vec<(String, (f32, f32), (f32, f32))> {
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

    pub fn clear_all_enemy_bullets(ctx: &mut Context, board: &mut Gameboard) {
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
}
