use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteAction};

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::fly::EnemyManager;
use crate::player::PlayerManager;

use crate::server::ServerEvent;

const EXPLOSION_DURATION: Duration = Duration::from_secs(2);
const RESPAWN_DELAY: Duration = Duration::from_millis(500);

static mut EXPLOSIONS: Option<HashMap<String, Instant>> = None;
static mut ENEMIES_CREATED: bool = false;
static mut PLAYER_RESPAWN_TIME: Option<Instant> = None;
static mut PLAYER_IS_DEAD: bool = false;
static mut SCORE: u32 = 0;
static mut LIVES: u32 = 4;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        unsafe {
            ENEMIES_CREATED = false;
            EXPLOSIONS = Some(HashMap::new());
            PLAYER_RESPAWN_TIME = None;
            PLAYER_IS_DEAD = false;
            SCORE = 0;
            LIVES = 4;
        }

        EnemyManager::initialize();
        PlayerManager::initialize();

        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));
        let player = PlayerManager::create_player(ctx);
        gameboard.insert_sprite(ctx, player);

        Self::create_score_display(ctx, &mut gameboard);

        gameboard
    }

    fn create_score_display(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let score = SCORE;
            let lives = LIVES;
            let wave = EnemyManager::get_wave_count();
        }
    }

    fn update_score_display(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let score = SCORE;
            let lives = LIVES;
            let wave = EnemyManager::get_wave_count();
        }
    }

    fn add_score(ctx: &mut Context, board: &mut Gameboard, points: u32) {
        unsafe {
            SCORE += points;
            let score = SCORE;
        }
        Self::update_score_display(ctx, board);
    }

    fn lose_life(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if LIVES > 0 {
                LIVES -= 1;
            }
            let lives = LIVES;
            println!("*** PLAYER HIT! Lives remaining: {} ***", lives);

            if LIVES == 0 {
                println!("*** GAME OVER! ***");
            }
        }

        Self::update_score_display(ctx, board);
    }

    fn respawn_player(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if LIVES > 0 {
                let player = PlayerManager::create_player(ctx);
                board.insert_sprite(ctx, player);
                println!("*** PLAYER RESPAWNED! ***");
            }

            PLAYER_IS_DEAD = false;
            PLAYER_RESPAWN_TIME = None;
        }
    }

    fn check_collision(
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

    fn spawn_explosion(ctx: &mut Context, board: &mut Gameboard, pos: (f32, f32)) {
        let id = format!("explosion_{}", uuid::Uuid::new_v4());
        let sprite = Sprite::new(
            ctx,
            &id,
            "explosion.png",
            (50.0, 50.0),
            (Offset::Static(pos.0), Offset::Static(pos.1)),
        );
        board.insert_sprite(ctx, sprite);

        unsafe {
            if let Some(ref mut explosions) = EXPLOSIONS {
                explosions.insert(id, Instant::now());
            }
        }
        println!("Spawned explosion at {:?}", pos);
    }

    fn remove_sprite_from_board(ctx: &mut Context, board: &mut Gameboard, sprite_id: &str) {
        if board.2.remove(sprite_id).is_some() {
            board.0.0.remove(sprite_id);

            if EnemyManager::is_enemy(sprite_id) {
                EnemyManager::remove_enemy_from_base_positions(sprite_id);
            }
        }
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            unsafe {
                if !ENEMIES_CREATED {
                    EnemyManager::create_enemies(ctx, board);
                    ENEMIES_CREATED = true;
                }
            }

            unsafe {
                if let Some(respawn_time) = PLAYER_RESPAWN_TIME {
                    if Instant::now().duration_since(respawn_time) >= RESPAWN_DELAY {
                        Self::respawn_player(ctx, board);
                    }
                }
            }

            unsafe {
                if !PLAYER_IS_DEAD {
                    PlayerManager::update_player_movement(ctx, board);
                }
            }

            EnemyManager::update_enemy_pulse(ctx, board);
            EnemyManager::update_enemy_shooting(ctx, board);

            let enemy_bullets_to_remove = EnemyManager::update_enemy_bullets(ctx, board);
            for bullet_id in &enemy_bullets_to_remove {
                Self::remove_sprite_from_board(ctx, board, bullet_id);
            }

            unsafe {
                if !PLAYER_IS_DEAD {
                    let active_enemy_bullets = EnemyManager::get_active_enemy_bullets(board, ctx);
                    let mut player_hit = false;
                    let mut player_hit_pos = (0.0, 0.0);
                    let mut enemy_bullets_to_remove_from_collision = Vec::new();

                    if let Some(player_sprite) = board.2.get_mut("player") {
                        let player_pos = player_sprite.position(ctx);
                        let player_size = *player_sprite.dimensions();

                        for (bullet_id, bullet_pos, bullet_size) in active_enemy_bullets {
                            if Self::check_collision(bullet_pos, bullet_size, player_pos, player_size) {
                                player_hit = true;
                                player_hit_pos = player_pos;
                                enemy_bullets_to_remove_from_collision.push(bullet_id);
                                break;
                            }
                        }
                    }

                    if player_hit {
                        for bullet_id in enemy_bullets_to_remove_from_collision {
                            Self::remove_sprite_from_board(ctx, board, &bullet_id);
                        }

                        Self::remove_sprite_from_board(ctx, board, "player");
                        Self::spawn_explosion(ctx, board, player_hit_pos);
                        Self::lose_life(ctx, board);

                        PLAYER_IS_DEAD = true;
                        PLAYER_RESPAWN_TIME = Some(Instant::now() + EXPLOSION_DURATION);

                        println!("PLAYER HIT!");
                    }
                }
            }

            let bullets_to_remove = PlayerManager::update_bullets(ctx, board);
            let active_bullets = PlayerManager::get_active_bullets(board, ctx);
            let active_enemy_bullets = EnemyManager::get_active_enemy_bullets(board, ctx);

            for bullet_id in &bullets_to_remove {
                Self::remove_sprite_from_board(ctx, board, bullet_id);
            }


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
                Self::spawn_explosion(ctx, board, explosion_pos);

                println!(
                    "Bullet–bullet collision: {} vs {} at {:?}",
                    player_bullet_id, enemy_bullet_id, explosion_pos
                );
            }

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

            if collisions_count > 0 {
                Self::add_score(ctx, board, collisions_count * 100);
            }

            for pos in explosions_to_spawn {
                Self::spawn_explosion(ctx, board, pos);
            }

            for sprite_id in sprites_to_remove {
                Self::remove_sprite_from_board(ctx, board, &sprite_id);
            }

            EnemyManager::check_and_manage_enemy_state(ctx, board);

            unsafe {
                if let Some(ref mut explosions) = EXPLOSIONS {
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
            }

            let sprite_ids: Vec<String> = board.2.keys().cloned().collect();
            for id in sprite_ids {
                if let Some(sprite) = board.2.get_mut(&id) {
                    let (x, y) = sprite.position(ctx);
                    if let Some(layout_pos) = board.0.0.get_mut(&id) {
                        layout_pos.0 = Offset::Static(x);
                        layout_pos.1 = Offset::Static(y);
                    }
                }
            }
        } else if let Some(keyboard_event) = event.downcast_ref::<KeyboardEvent>() {
            unsafe {
                if !PLAYER_IS_DEAD {
                    PlayerManager::handle_keyboard_input(ctx, board, keyboard_event);
                }
            }
        }
        true
    }
}
