use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{AspectRatio, Sprite, Gameboard, SpriteAction};

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub use crate::fly_manager::EnemyManager;
pub use crate::fly_state::{EnemyState, EnemyGlobalState};
pub use crate::collision::CollisionManager;

use crate::player::PlayerManager;
use crate::server::{ServerEvent, GameServer, ServerEventHandler, GameAction};

const EXPLOSION_DURATION: Duration = Duration::from_secs(2);
const RESPAWN_DELAY: Duration = Duration::from_millis(500);

static mut EXPLOSIONS: Option<HashMap<String, Instant>> = None;
static mut ENEMIES_CREATED: bool = false;
static mut PLAYER_RESPAWN_TIME: Option<Instant> = None;
static mut PLAYER_IS_DEAD: bool = false;
static mut SCORE: u32 = 0;
static mut LIVES: u32 = 4;
static mut SERVER_EVENT_HANDLER: Option<ServerEventHandler> = None;
static mut GAME_SERVER: Option<GameServer> = None;

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
            LIVES = 4;
        }

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

    fn remove_sprite_from_board(ctx: &mut Context, board: &mut Gameboard, sprite_id: &str) {
        if board.2.remove(sprite_id).is_some() {
            board.0.0.remove(sprite_id);

            if EnemyManager::is_enemy(sprite_id) {
                EnemyManager::remove_enemy_from_base_positions(sprite_id);
            }
        }
    }
    
    fn handle_server_input(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if let Some(ref event_handler) = SERVER_EVENT_HANDLER {
                if let Some(action) = event_handler.process_events_for_game() {
                    match action {
                        GameAction::MoveRight => {
                            println!("🎮 Server input: Move Right");
                            PlayerManager::handle_server_move_right(ctx, board);
                        }
                        GameAction::MoveLeft => {
                            println!("🎮 Server input: Move Left");
                            PlayerManager::handle_server_move_left(ctx, board);
                        }
                        GameAction::Shoot => {
                            println!("🎮 Server input: Shoot");
                            PlayerManager::handle_server_shoot(ctx, board);
                        }
                    }
                }
            }
        }
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            // Handle server input first
            Self::handle_server_input(ctx, board);
            
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

            // Handle player-enemy bullet collisions
            unsafe {
                if !PLAYER_IS_DEAD {
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

            // Update player bullets
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