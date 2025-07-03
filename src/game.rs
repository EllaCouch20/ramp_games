use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};
use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteAction};

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub use crate::fly::fly_manager::EnemyManager;
pub use crate::fly::fly_state::{EnemyState, EnemyGlobalState};
pub use crate::collision::CollisionManager;

use crate::player::{PlayerManager, PlayerLives, LivesDisplayInfo, PlayerState, MovementDirection, KeysHeld, ServerMovement};
use crate::server::{ServerEvent, GameServer, ServerEventHandler, GameAction};

use crate::settings::GameSettings;

const EXPLOSION_DURATION: Duration = Duration::from_secs(2);
const RESPAWN_DELAY: Duration = Duration::from_millis(500);
const LIFE_SPRITE_SIZE: (f32, f32) = (30.0, 30.0);
const LIFE_SPRITE_SPACING: f32 = 35.0;
const LIFE_SPRITE_START_X: f32 = 20.0;
const LIFE_SPRITE_Y: f32 = 20.0; // Bottom left area

static mut EXPLOSIONS: Option<HashMap<String, Instant>> = None;
static mut ENEMIES_CREATED: bool = false;
static mut PLAYER_RESPAWN_TIME: Option<Instant> = None;
static mut PLAYER_IS_DEAD: bool = false;
static mut SCORE: u32 = 0;
static mut SERVER_EVENT_HANDLER: Option<ServerEventHandler> = None;
static mut GAME_SERVER: Option<GameServer> = None;
static mut GAME_SETTINGS: Option<GameSettings> = None;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        Self::initialize_game_state();
        Self::create_gameboard(ctx)
    }
    
    pub fn new_with_server(ctx: &mut Context, server: GameServer, event_handler: ServerEventHandler) -> Gameboard {
        Self::initialize_game_state();
        
        unsafe {
            GAME_SERVER = Some(server);
            SERVER_EVENT_HANDLER = Some(event_handler);
        }
        
        Self::create_gameboard(ctx)
    }
    
    fn initialize_game_state() {
        unsafe {
            ENEMIES_CREATED = false;
            EXPLOSIONS = Some(HashMap::new());
            PLAYER_RESPAWN_TIME = None;
            PLAYER_IS_DEAD = false;
            SCORE = 0;
            GAME_SETTINGS = Some(GameSettings::new()); 
        }

        PlayerLives::initialize_with_lives(4);
        EnemyManager::initialize();
        PlayerManager::initialize();
    }
    
    fn create_gameboard(ctx: &mut Context) -> Gameboard {
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));
        let player = PlayerManager::create_player(ctx);
        gameboard.insert_sprite(ctx, player);

        Self::create_score_display(ctx, &mut gameboard);

        gameboard
    }

    fn create_score_display(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let score = SCORE;
            let lives_info = PlayerLives::get_display_info();
            let wave = EnemyManager::get_wave_count();
            
            // Create life sprites for each remaining life
            Self::create_life_sprites(ctx, board, lives_info.lives);
        }
    }

    fn update_score_display(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let score = SCORE;
            let lives_info = PlayerLives::get_display_info();
            let wave = EnemyManager::get_wave_count();
            
            // Update life sprites
            Self::update_life_sprites(ctx, board, lives_info.lives);
        }
    }

    fn create_life_sprites(ctx: &mut Context, board: &mut Gameboard, lives: u32) {
        // Remove any existing life sprites first
        Self::remove_all_life_sprites(ctx, board);
        
        // Create new life sprites
        for i in 0..lives {
            let life_sprite_id = format!("life_{}", i);
            let x_pos = LIFE_SPRITE_START_X + (i as f32 * LIFE_SPRITE_SPACING);
            
            let mut life_sprite = Sprite::new(
                ctx,
                &life_sprite_id,
                "spaceship_blue.png", // Same sprite as player
                LIFE_SPRITE_SIZE,
                (Offset::Static(x_pos), Offset::Static(LIFE_SPRITE_Y))
            );
            
            // Set adjustments to 0 for precise positioning
            life_sprite.adjustments().0 = 0.0;
            life_sprite.adjustments().1 = 0.0;
            
            board.insert_sprite(ctx, life_sprite);
        }
    }

    fn update_life_sprites(ctx: &mut Context, board: &mut Gameboard, lives: u32) {
        // Get current life sprite count
        let current_life_sprites: Vec<String> = board.2.keys()
            .filter(|id| id.starts_with("life_"))
            .cloned()
            .collect();
        
        let current_count = current_life_sprites.len() as u32;
        
        // Only update if the count has changed
        if current_count != lives {
            Self::create_life_sprites(ctx, board, lives);
        }
    }

    fn remove_all_life_sprites(ctx: &mut Context, board: &mut Gameboard) {
        let life_sprite_ids: Vec<String> = board.2.keys()
            .filter(|id| id.starts_with("life_"))
            .cloned()
            .collect();
        
        for life_id in life_sprite_ids {
            Self::remove_sprite_from_board(ctx, board, &life_id);
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
        PlayerLives::handle_player_death();
        let lives_info = PlayerLives::get_display_info();
        println!("PLAYER died! Lives remaining: {}", lives_info.lives);

        if PlayerLives::is_game_over() {
            println!("GAME OVER");
            // Remove all life sprites when game is over
            Self::remove_all_life_sprites(ctx, board);
        } else {
            // Update the life sprites display
            Self::update_score_display(ctx, board);
        }
    }

    fn respawn_player(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let can_respawn = !PlayerLives::is_game_over();
            
            if can_respawn {
                let player = PlayerManager::create_player(ctx);
                board.insert_sprite(ctx, player);
                println!("*** PLAYER RESPAWNED! ***");
            }

            PLAYER_IS_DEAD = false;
            PLAYER_RESPAWN_TIME = None;
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

    fn is_life_sprite(sprite_id: &str) -> bool {
        sprite_id.starts_with("life_")
    }
    
    fn handle_server_input(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if let Some(ref event_handler) = SERVER_EVENT_HANDLER {
                if let Some(action) = event_handler.process_events_for_game() {
                    match action {
                        GameAction::MoveRight => {
                            println!("Server input: Move Right");
                            PlayerManager::handle_server_move_right(ctx, board);
                        }
                        GameAction::MoveLeft => {
                            println!("Server input: Move Left");
                            PlayerManager::handle_server_move_left(ctx, board);
                        }
                        GameAction::Shoot => {
                            println!("Server input: Shoot");
                            PlayerManager::handle_server_shoot(ctx, board);
                        }
                    }
                }
            }
        }
    }

    pub fn get_game_settings() -> Option<GameSettings> {
        unsafe {
            let settings_ptr = std::ptr::addr_of!(GAME_SETTINGS);
            (*settings_ptr).clone()
        }
    }

    pub fn update_game_settings<F>(updater: F) 
    where
        F: FnOnce(&mut GameSettings),
    {
        unsafe {
            let settings_ptr = std::ptr::addr_of_mut!(GAME_SETTINGS);
            if let Some(settings) = &mut *settings_ptr {
                updater(settings);
            }
        }
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {

            Self::handle_server_input(ctx, board);
            
            unsafe {
                if !ENEMIES_CREATED {
                    EnemyManager::create_enemies(ctx, board);
                    ENEMIES_CREATED = true;
                }
            }

            PlayerLives::update(ctx, board);

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
                    let settings_ptr = std::ptr::addr_of!(GAME_SETTINGS);
                    let player_invincible = (*settings_ptr).as_ref().map(|s| s.player_invincible).unwrap_or(false);
                    
                    if !player_invincible {
                        if let Some(ref mut explosions) = EXPLOSIONS {
                            let (player_hit, player_hit_pos) = CollisionManager::handle_player_enemy_bullet_collisions(
                                ctx, board, explosions
                            );

                            if player_hit {
                                Self::remove_sprite_from_board(ctx, board, "player");
                                CollisionManager::spawn_explosion(ctx, board, player_hit_pos, explosions);
                                Self::lose_life(ctx, board);

                                PLAYER_IS_DEAD = true;
                                PLAYER_RESPAWN_TIME = Some(Instant::now() + EXPLOSION_DURATION);

                                println!("PLAYER HIT!");
                            }
                        }
                    }
                }
            }

            let bullets_to_remove = PlayerManager::update_bullets(ctx, board);
            for bullet_id in &bullets_to_remove {
                Self::remove_sprite_from_board(ctx, board, bullet_id);
            }

            unsafe {
                if let Some(ref mut explosions) = EXPLOSIONS {
                    CollisionManager::handle_bullet_bullet_collisions(ctx, board, explosions);

                    let collisions_count = CollisionManager::handle_player_bullet_enemy_collisions(
                        ctx, board, explosions
                    );

                    if collisions_count > 0 {
                        Self::add_score(ctx, board, collisions_count * 100);
                    }

                    CollisionManager::update_explosions(ctx, board, explosions);
                }
            }

            EnemyManager::check_and_manage_enemy_state(ctx, board);

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

impl Drop for Galaga {
    fn drop(&mut self) {
        unsafe {
            let server_ptr = std::ptr::addr_of_mut!(GAME_SERVER);
            if let Some(mut server) = (*server_ptr).take() {
                server.stop();
                println!("Game server stopped during cleanup");
            }
        }
    }
}