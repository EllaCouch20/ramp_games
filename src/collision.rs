use pelican_ui::Context;
use pelican_game_engine::{Gameboard, Sprite};
use std::time::Instant;
use std::collections::HashMap;
use std::time::Duration;
use pelican_ui_std::Offset;

use crate::player::PlayerManager;
use crate::fly_manager::EnemyManager;

const EXPLOSION_DURATION: Duration = Duration::from_secs(2);

pub struct CollisionManager;

impl CollisionManager {
    pub fn check_collision(
        sprite1_pos: (f32, f32),
        sprite1_size: (f32, f32),
        sprite2_pos: (f32, f32),
        sprite2_size: (f32, f32),
    ) -> bool {
        let (x1, y1) = sprite1_pos;
        let (w1, h1) = sprite1_size;
        let (x2, y2) = sprite2_pos;
        let (w2, h2) = sprite2_size;

        x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
    }

    pub fn handle_player_enemy_bullet_collisions(
        ctx: &mut Context,
        board: &mut Gameboard,
        explosions: &mut HashMap<String, Instant>,
    ) -> (bool, (f32, f32)) {
        let active_enemy_bullets = EnemyManager::get_active_enemy_bullets(board, ctx);
        let mut player_hit = false;
        let mut player_hit_pos = (0.0, 0.0);
        let mut enemy_bullets_to_remove = Vec::new();

        if let Some(player_sprite) = board.2.get_mut("player") {
            let player_pos = player_sprite.position(ctx);
            let player_size = *player_sprite.dimensions();

            for (bullet_id, bullet_pos, bullet_size) in active_enemy_bullets {
                if Self::check_collision(bullet_pos, bullet_size, player_pos, player_size) {
                    player_hit = true;
                    player_hit_pos = player_pos;
                    enemy_bullets_to_remove.push(bullet_id);
                    break;
                }
            }
        }

        for bullet_id in enemy_bullets_to_remove {
            Self::remove_sprite_from_board(ctx, board, &bullet_id);
        }

        (player_hit, player_hit_pos)
    }

    pub fn handle_bullet_bullet_collisions(
        ctx: &mut Context,
        board: &mut Gameboard,
        explosions: &mut HashMap<String, Instant>,
    ) {
        let active_bullets = PlayerManager::get_active_bullets(board, ctx);
        let active_enemy_bullets = EnemyManager::get_active_enemy_bullets(board, ctx);
        let mut bullet_bullet_collisions = Vec::new();

        for (player_bullet_id, player_pos, player_size) in &active_bullets {
            for (enemy_bullet_id, enemy_pos, enemy_size) in &active_enemy_bullets {
                if Self::check_collision(*player_pos, *player_size, *enemy_pos, *enemy_size) {
                    bullet_bullet_collisions.push((
                        player_bullet_id.clone(),
                        enemy_bullet_id.clone(),
                        (
                            (player_pos.0 + enemy_pos.0) / 2.0,
                            (player_pos.1 + enemy_pos.1) / 2.0,
                        ),
                    ));
                }
            }
        }

        for (player_bullet_id, enemy_bullet_id, explosion_pos) in bullet_bullet_collisions {
            Self::remove_sprite_from_board(ctx, board, &player_bullet_id);
            Self::remove_sprite_from_board(ctx, board, &enemy_bullet_id);
            Self::spawn_explosion(ctx, board, explosion_pos, explosions);

            println!(
                "Bullet–bullet collision: {} vs {} at {:?}",
                player_bullet_id, enemy_bullet_id, explosion_pos
            );
        }
    }

    pub fn handle_player_bullet_enemy_collisions(
        ctx: &mut Context,
        board: &mut Gameboard,
        explosions: &mut HashMap<String, Instant>,
    ) -> u32 {
        let active_bullets = PlayerManager::get_active_bullets(board, ctx);
        let mut sprites_to_remove = Vec::new();
        let mut explosions_to_spawn = Vec::new();
        let mut collisions_count = 0;

        for (bullet_id, bullet_pos, bullet_size) in active_bullets {
            for (enemy_id, enemy_sprite) in board.2.iter_mut() {
                if EnemyManager::is_enemy(enemy_id) {
                    let enemy_pos = enemy_sprite.position(ctx);
                    let enemy_size = *enemy_sprite.dimensions();

                    if Self::check_collision(bullet_pos, bullet_size, enemy_pos, enemy_size) {
                        explosions_to_spawn.push(enemy_pos);
                        sprites_to_remove.push(bullet_id.clone());
                        sprites_to_remove.push(enemy_id.clone());
                        collisions_count += 1;
                        break;
                    }
                }
            }
        }

        for pos in explosions_to_spawn {
            Self::spawn_explosion(ctx, board, pos, explosions);
        }

        for sprite_id in sprites_to_remove {
            Self::remove_sprite_from_board(ctx, board, &sprite_id);
        }

        collisions_count
    }

    pub fn spawn_explosion(
        ctx: &mut Context,
        board: &mut Gameboard,
        pos: (f32, f32),
        explosions: &mut HashMap<String, Instant>,
    ) {
        let id = format!("explosion_{}", uuid::Uuid::new_v4());
        let sprite = Sprite::new(
            ctx,
            &id,
            "explosion.png",
            (50.0, 50.0),
            (Offset::Static(pos.0), Offset::Static(pos.1)),
        );
        board.insert_sprite(ctx, sprite);
        explosions.insert(id, Instant::now());
        println!("Spawned explosion at {:?}", pos);
    }

    pub fn update_explosions(
        ctx: &mut Context,
        board: &mut Gameboard,
        explosions: &mut HashMap<String, Instant>,
    ) {
        let now = Instant::now();
        let mut expired_explosions = Vec::new();

        for (id, time) in explosions.iter() {
            if now.duration_since(*time) >= EXPLOSION_DURATION {
                expired_explosions.push(id.clone());
            }
        }

        for id in expired_explosions {
            Self::remove_sprite_from_board(ctx, board, &id);
            explosions.remove(&id);
        }
    }

    fn remove_sprite_from_board(ctx: &mut Context, board: &mut Gameboard, sprite_id: &str) {
        if board.2.remove(sprite_id).is_some() {
            board.0.0.remove(sprite_id);

            if EnemyManager::is_enemy(sprite_id) {
                EnemyManager::remove_enemy_from_base_positions(sprite_id);
            }
        }
    }
}