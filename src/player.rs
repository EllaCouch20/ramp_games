use pelican_ui::{Context};
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard, SpriteAction};
use pelican_ui::events::{KeyboardEvent, KeyboardState, Key, NamedKey};
use std::time::{Duration, Instant};

const STEP: f32 = 1.5; 
const BULLET_SPEED: f32 = 8.0;
const SHOOT_COOLDOWN: Duration = Duration::from_millis(200);
const MOVEMENT_SPEED: f32 = 300.0; 

#[derive(Clone, Copy, Debug)]
enum PlayerState {
    Idle { last_shot: Option<Instant> },
    MovingLeft { last_shot: Option<Instant>, speed: f32 },
    MovingRight { last_shot: Option<Instant>, speed: f32 },
    MovingBoth { last_shot: Option<Instant>, left_speed: f32, right_speed: f32 },
    Shooting { direction: MovementDirection, shot_time: Instant },
    Destroyed,
}

#[derive(Clone, Copy, Debug)]
enum MovementDirection {
    None,
    Left,
    Right,
    Both,
}

#[derive(Clone, Copy)]
struct KeysHeld {
    left: bool,
    right: bool,
}

impl KeysHeld {
    const fn new() -> Self {
        Self { left: false, right: false }
    }
    
    fn to_direction(&self) -> MovementDirection {
        match (self.left, self.right) {
            (true, true) => MovementDirection::Both,
            (true, false) => MovementDirection::Left,
            (false, true) => MovementDirection::Right,
            (false, false) => MovementDirection::None,
        }
    }
}

static mut PLAYER_STATE: PlayerState = PlayerState::Idle { last_shot: None };
static mut KEYS_HELD: KeysHeld = KeysHeld::new();
static mut LAST_UPDATE_TIME: Option<Instant> = None;

pub struct PlayerManager;

impl PlayerManager {
    pub fn initialize() {
        unsafe {
            PLAYER_STATE = PlayerState::Idle { last_shot: None };
            KEYS_HELD = KeysHeld::new();
            LAST_UPDATE_TIME = Some(Instant::now());
        }
    }

    pub fn create_player(ctx: &mut Context) -> Sprite {
        let mut player = Sprite::new(ctx, "player", "spaceship_blue.png", (50.0, 50.0), (Offset::Center, Offset::End));
        player.adjustments().0 = 0.0;
        player.adjustments().1 = 0.0;
        player
    }

    pub fn handle_keyboard_input(ctx: &mut Context, board: &mut Gameboard, keyboard_event: &KeyboardEvent) -> bool {
        let mut handled = false;
        
        unsafe {
            // Update key states
            match keyboard_event {
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowLeft) } => {
                    KEYS_HELD.left = true;
                    handled = true;
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowLeft) } => {
                    KEYS_HELD.left = false;
                    handled = true;
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowRight) } => {
                    KEYS_HELD.right = true;
                    handled = true;
                }
                KeyboardEvent { state: KeyboardState::Released, key: Key::Named(NamedKey::ArrowRight) } => {
                    KEYS_HELD.right = false;
                    handled = true;
                }
                KeyboardEvent { state: KeyboardState::Pressed, key: Key::Named(NamedKey::ArrowUp) } => {
                    Self::handle_shooting(ctx, board);
                    handled = true;
                }
                _ => {}
            }
            
            Self::update_player_state();
        }
        
        handled
    }

    // Clever match statement to handle state transitions
    fn update_player_state() {
        unsafe {
            let keys_copy = KEYS_HELD;
            let direction = keys_copy.to_direction();
            let now = Instant::now();
            
            PLAYER_STATE = match (PLAYER_STATE, direction) {
                (PlayerState::Idle { last_shot }, MovementDirection::Left) => 
                    PlayerState::MovingLeft { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::Idle { last_shot }, MovementDirection::Right) => 
                    PlayerState::MovingRight { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::Idle { last_shot }, MovementDirection::Both) => 
                    PlayerState::MovingBoth { last_shot, left_speed: MOVEMENT_SPEED, right_speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingLeft { last_shot, .. }, MovementDirection::None) => 
                    PlayerState::Idle { last_shot },
                
                (PlayerState::MovingLeft { last_shot, .. }, MovementDirection::Right) => 
                    PlayerState::MovingRight { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingLeft { last_shot, .. }, MovementDirection::Both) => 
                    PlayerState::MovingBoth { last_shot, left_speed: MOVEMENT_SPEED, right_speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingRight { last_shot, .. }, MovementDirection::None) => 
                    PlayerState::Idle { last_shot },
                
                (PlayerState::MovingRight { last_shot, .. }, MovementDirection::Left) => 
                    PlayerState::MovingLeft { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingRight { last_shot, .. }, MovementDirection::Both) => 
                    PlayerState::MovingBoth { last_shot, left_speed: MOVEMENT_SPEED, right_speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingBoth { last_shot, .. }, MovementDirection::None) => 
                    PlayerState::Idle { last_shot },
                
                (PlayerState::MovingBoth { last_shot, .. }, MovementDirection::Left) => 
                    PlayerState::MovingLeft { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::MovingBoth { last_shot, .. }, MovementDirection::Right) => 
                    PlayerState::MovingRight { last_shot, speed: MOVEMENT_SPEED },
                
                (PlayerState::Shooting { direction, shot_time }, _) => {
                    if now.duration_since(shot_time) > Duration::from_millis(50) {
                        match direction {
                            MovementDirection::None => PlayerState::Idle { last_shot: Some(shot_time) },
                            MovementDirection::Left => PlayerState::MovingLeft { last_shot: Some(shot_time), speed: MOVEMENT_SPEED },
                            MovementDirection::Right => PlayerState::MovingRight { last_shot: Some(shot_time), speed: MOVEMENT_SPEED },
                            MovementDirection::Both => PlayerState::MovingBoth { last_shot: Some(shot_time), left_speed: MOVEMENT_SPEED, right_speed: MOVEMENT_SPEED },
                        }
                    } else {
                        PlayerState::Shooting { direction, shot_time }
                    }
                }
                
                (PlayerState::Destroyed, _) => PlayerState::Destroyed,
                
                (current_state, _) => current_state,
            };
        }
    }

    pub fn update_player_movement(ctx: &mut Context, board: &mut Gameboard) {
        if !board.2.contains_key("player") {
            return;
        }

        unsafe {
            Self::handle_movement_by_state(ctx, board);
        }
    }

    // Clever match to handle movement based on current state
    fn handle_movement_by_state(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let (maxw, _) = board.0.size(ctx);
            
            if let Some(sprite) = board.2.get_mut("player") {
                let current_pos = sprite.position(ctx).0;
                
                match PLAYER_STATE {
                    PlayerState::MovingLeft { speed, .. } => {
                        if current_pos > 5.0 {
                            sprite.adjustments().0 -= STEP;
                        }
                    }
                    
                    PlayerState::MovingRight { speed, .. } => {
                        if current_pos < maxw - sprite.dimensions().0 - 5.0 {
                            sprite.adjustments().0 += STEP;
                        }
                    }
                    
                    PlayerState::MovingBoth { left_speed, right_speed, .. } => {
                        
                    }
                    
                    PlayerState::Shooting { direction, .. } => {
                        match direction {
                            MovementDirection::Left if current_pos > 5.0 => {
                                sprite.adjustments().0 -= STEP * 0.5;
                            }
                            MovementDirection::Right if current_pos < maxw - sprite.dimensions().0 - 5.0 => {
                                sprite.adjustments().0 += STEP * 0.5;
                            }
                            _ => {}
                        }
                    }
                    
                    PlayerState::Idle { .. } | PlayerState::Destroyed => {
                        
                    }
                }
            }
        }
    }

    pub fn handle_player_action(ctx: &mut Context, board: &mut Gameboard, action: SpriteAction) {
        if !board.2.contains_key("player") {
            return;
        }
        
        match action {
            SpriteAction::Shoot => {
                Self::handle_shooting(ctx, board);
            }
            _ => {}
        }
    }

    fn handle_shooting(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let can_shoot = match PLAYER_STATE {
                PlayerState::Idle { last_shot } | 
                PlayerState::MovingLeft { last_shot, .. } | 
                PlayerState::MovingRight { last_shot, .. } | 
                PlayerState::MovingBoth { last_shot, .. } => {
                    if let Some(last) = last_shot {
                        Instant::now().duration_since(last) >= SHOOT_COOLDOWN
                    } else {
                        true
                    }
                }
                PlayerState::Shooting { shot_time, .. } => {
                    Instant::now().duration_since(shot_time) >= SHOOT_COOLDOWN
                }
                PlayerState::Destroyed => false,
            };
            
            if can_shoot {
                let player_info = if let Some(sprite) = board.2.get_mut("player") {
                    Some((sprite.position(ctx), *sprite.dimensions()))
                } else {
                    None
                };

                if let Some((pos, size)) = player_info {
                    Self::shoot(ctx, board, pos, size);
                    
                    let keys_copy = KEYS_HELD;
                    let current_direction = keys_copy.to_direction();
                    PLAYER_STATE = PlayerState::Shooting { 
                        direction: current_direction, 
                        shot_time: Instant::now() 
                    };
                }
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

    pub fn get_player_state() -> PlayerState {
        unsafe { PLAYER_STATE }
    }

    pub fn destroy_player() {
        unsafe {
            PLAYER_STATE = PlayerState::Destroyed;
        }
    }

    pub fn is_player_destroyed() -> bool {
        unsafe {
            matches!(PLAYER_STATE, PlayerState::Destroyed)
        }
    }

    pub fn update_bullets(ctx: &mut Context, board: &mut Gameboard) -> Vec<String> {
        let mut bullets_to_remove = Vec::new();

        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("bullet_") {
                sprite.adjustments().1 -= BULLET_SPEED;
                let pos = sprite.position(ctx);

                if pos.1 < -50.0 {
                    bullets_to_remove.push(id.clone());
                }
            }
        }

        bullets_to_remove
    }

    pub fn get_active_bullets(board: &mut Gameboard, ctx: &mut Context) -> Vec<(String, (f32, f32), (f32, f32))> {
        let mut active_bullets = Vec::new();

        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("bullet_") {
                let pos = sprite.position(ctx);
                let size = *sprite.dimensions();
                active_bullets.push((id.clone(), pos, size));
            }
        }

        active_bullets
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
}