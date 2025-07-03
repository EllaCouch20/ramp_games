use pelican_ui::{Context};
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard, SpriteAction};
use pelican_ui::events::{KeyboardEvent, KeyboardState, Key, NamedKey};
use std::time::Instant;

use super::{PlayerState, MovementDirection, KeysHeld, ServerMovement};
use super::{BULLET_SPEED, SHOOT_COOLDOWN, MOVEMENT_SPEED, SERVER_MOVEMENT_DURATION};

static mut PLAYER_STATE: PlayerState = PlayerState::Idle { last_shot: None };
static mut KEYS_HELD: KeysHeld = KeysHeld::new();
static mut SERVER_MOVEMENT: Option<ServerMovement> = None;

pub struct PlayerManager;

impl PlayerManager {
    pub fn initialize() {
        unsafe {
            PLAYER_STATE = PlayerState::Idle { last_shot: None };
            KEYS_HELD = KeysHeld::new();
            SERVER_MOVEMENT = None;
        }
    }

    pub fn create_player(ctx: &mut Context) -> Sprite {
        let mut player = Sprite::new(ctx, "player", "spaceship_blue.png", (50.0, 50.0), (Offset::Center, Offset::End));
        player.adjustments().0 = 0.0;
        player.adjustments().1 = 0.0;
        player
    }

    pub fn handle_keyboard_input(ctx: &mut Context, board: &mut Gameboard, event: &KeyboardEvent) -> bool {
        unsafe {
            match event {
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowLeft) } => {
                    KEYS_HELD.left = true;
                    Self::update_player_state();
                    true
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowLeft) } => {
                    KEYS_HELD.left = false;
                    Self::update_player_state();
                    true
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowRight) } => {
                    KEYS_HELD.right = true;
                    Self::update_player_state();
                    true
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowRight) } => {
                    KEYS_HELD.right = false;
                    Self::update_player_state();
                    true
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowUp) } => {
                    Self::handle_shooting(ctx, board);
                    true
                }
                _ => false
            }
        }
    }

    pub fn handle_server_move_right(ctx: &mut Context, board: &mut Gameboard) {
        Self::set_server_movement(MovementDirection::Right);
        println!("Server: Move Right activated");
    }
    
    pub fn handle_server_move_left(ctx: &mut Context, board: &mut Gameboard) {
        Self::set_server_movement(MovementDirection::Left);
        println!("Server: Move Left activated");
    }
    
    pub fn handle_server_shoot(ctx: &mut Context, board: &mut Gameboard) {
        Self::handle_shooting(ctx, board);
        println!("Server: Shoot activated");
    }

    pub fn handle_player_action(ctx: &mut Context, board: &mut Gameboard, action: SpriteAction) {
        if matches!(action, SpriteAction::Shoot) && board.2.contains_key("player") {
            Self::handle_shooting(ctx, board);
        }
    }

    fn set_server_movement(direction: MovementDirection) {
        unsafe {
            SERVER_MOVEMENT = Some(ServerMovement {
                direction,
                start_time: Instant::now(),
            });
        }
    }

    fn can_shoot() -> bool {
        unsafe {
            let last_shot = match PLAYER_STATE {
                PlayerState::Idle { last_shot } |
                PlayerState::MovingLeft { last_shot, .. } |
                PlayerState::MovingRight { last_shot, .. } |
                PlayerState::MovingBoth { last_shot, .. } => last_shot,
                PlayerState::Shooting { shot_time, .. } => Some(shot_time),
                PlayerState::Destroyed => return false,
            };

            last_shot.map_or(true, |t| Instant::now().duration_since(t) >= SHOOT_COOLDOWN)
        }
    }

    fn handle_shooting(ctx: &mut Context, board: &mut Gameboard) {
        if !Self::can_shoot() { return; }

        let player_info = board.2.get_mut("player")
            .map(|sprite| (sprite.position(ctx), *sprite.dimensions()));

        if let Some((pos, size)) = player_info {
            Self::shoot(ctx, board, pos, size);

            unsafe {
                let direction = Self::get_current_direction();
                PLAYER_STATE = PlayerState::Shooting {
                    direction,
                    shot_time: Instant::now()
                };
            }
        }
    }

    fn shoot(ctx: &mut Context, board: &mut Gameboard, player_pos: (f32, f32), player_size: (f32, f32)) {
        let b_size = (15.0, 15.0);
        let (x, y) = player_pos;
        let bullet_id = format!("bullet_{}", uuid::Uuid::new_v4());
        let bullet = Sprite::new(
            ctx,
            &bullet_id,
            "bullet_blue.png",
            b_size,
            (
                Offset::Static(x + ((player_size.0 - b_size.0) / 2.0)),
                Offset::Static(y - 20.0),
            ),
        );
        board.insert_sprite(ctx, bullet);
    }

    fn get_current_direction() -> MovementDirection {
        unsafe {
            match Self::get_server_movement() {
                MovementDirection::None => {
                    let keys_copy = KEYS_HELD;
                    keys_copy.to_direction()
                },
                server_dir => server_dir,
            }
        }
    }

    fn get_server_movement() -> MovementDirection {
        unsafe {
            if let Some(server_mov) = SERVER_MOVEMENT {
                if Instant::now().duration_since(server_mov.start_time) < SERVER_MOVEMENT_DURATION {
                    return server_mov.direction;
                } else {
                    SERVER_MOVEMENT = None;
                }
            }
            MovementDirection::None
        }
    }

    fn update_player_state() {
        unsafe {
            let direction = Self::get_current_direction();
            
            PLAYER_STATE = match (PLAYER_STATE, direction) {
                (PlayerState::Destroyed, _) => PlayerState::Destroyed,
                
                (PlayerState::Shooting { shot_time, .. }, dir) => {
                    Self::state_from_direction(dir, Some(shot_time))
                }
                
                (state, dir) => {
                    let last_shot = match state {
                        PlayerState::Idle { last_shot } |
                        PlayerState::MovingLeft { last_shot, .. } |
                        PlayerState::MovingRight { last_shot, .. } |
                        PlayerState::MovingBoth { last_shot, .. } => last_shot,
                        _ => None,
                    };
                    Self::state_from_direction(dir, last_shot)
                }
            };
        }
    }

    fn state_from_direction(direction: MovementDirection, last_shot: Option<Instant>) -> PlayerState {
        match direction {
            MovementDirection::None => PlayerState::Idle { last_shot },
            MovementDirection::Left => PlayerState::MovingLeft { last_shot, speed: MOVEMENT_SPEED },
            MovementDirection::Right => PlayerState::MovingRight { last_shot, speed: MOVEMENT_SPEED },
            MovementDirection::Both => PlayerState::MovingBoth { 
                last_shot, 
                left_speed: MOVEMENT_SPEED, 
                right_speed: MOVEMENT_SPEED 
            },
        }
    }

    pub fn update_bullets(ctx: &mut Context, board: &mut Gameboard) -> Vec<String> {
        board.2.iter_mut()
            .filter_map(|(id, sprite)| {
                if id.starts_with("bullet_") {
                    sprite.adjustments().1 -= BULLET_SPEED;
                    if sprite.position(ctx).1 < -50.0 {
                        Some(id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_active_bullets(board: &mut Gameboard, ctx: &mut Context) -> Vec<(String, (f32, f32), (f32, f32))> {
        board.2.iter_mut()
            .filter_map(|(id, sprite)| {
                if id.starts_with("bullet_") {
                    Some((id.clone(), sprite.position(ctx), *sprite.dimensions()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_player_state() -> PlayerState {
        unsafe { PLAYER_STATE }
    }

    pub fn destroy_player() {
        unsafe { PLAYER_STATE = PlayerState::Destroyed; }
    }

    pub fn is_player_destroyed() -> bool {
        unsafe { matches!(PLAYER_STATE, PlayerState::Destroyed) }
    }

    pub fn is_bullet(sprite_id: &str) -> bool {
        sprite_id.starts_with("bullet_")
    }

    pub fn is_player(sprite_id: &str) -> bool {
        sprite_id == "player"
    }

    pub fn player_exists(board: &Gameboard) -> bool {
        board.2.contains_key("player")
    }

    pub fn update_player_movement(ctx: &mut Context, board: &mut Gameboard) {
        if board.2.contains_key("player") {
            unsafe {
                Self::update_player_state();
                crate::player::movement::handle_movement_by_state(ctx, board, PLAYER_STATE);
            }
        }
    }
}