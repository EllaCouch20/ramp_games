use pelican_ui::{Context};
use pelican_ui_std::Offset;
use pelican_game_engine::{Sprite, Gameboard, SpriteAction};
use pelican_ui::events::{KeyboardEvent, KeyboardState, Key, NamedKey};
use std::time::{Duration, Instant};

const STEP: f32 = 1.5; 
const BULLET_SPEED: f32 = 8.0;
const SHOOT_COOLDOWN: Duration = Duration::from_millis(500);

const MOVEMENT_SPEED: f32 = 300.0; 

static mut LAST_SHOT_TIME: Option<Instant> = None;
static mut KEYS_HELD: KeysHeld = KeysHeld::new();
static mut LAST_UPDATE_TIME: Option<Instant> = None;

#[derive(Clone, Copy)]
struct KeysHeld {
    left: bool,
    right: bool,
}

impl KeysHeld {
    const fn new() -> Self {
        Self {
            left: false,
            right: false,
        }
    }
}

pub struct PlayerManager;

impl PlayerManager {
    pub fn initialize() {
        unsafe {
            LAST_SHOT_TIME = None;
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
        }
        
        handled
    }

    pub fn update_player_movement(ctx: &mut Context, board: &mut Gameboard) {
        if !board.2.contains_key("player") {
            return;
        }

        unsafe {
            Self::update_with_fixed_steps(ctx, board);
        }
    }

    fn update_with_fixed_steps(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if !KEYS_HELD.left && !KEYS_HELD.right {
                return;
            }

            let (maxw, _) = board.0.size(ctx);
            
            if let Some(sprite) = board.2.get_mut("player") {
                let current_pos = sprite.position(ctx).0;
                
                if KEYS_HELD.left && current_pos > 5.0 {
                    sprite.adjustments().0 -= STEP;
                }
                if KEYS_HELD.right && current_pos < maxw - sprite.dimensions().0 - 5.0 {
                    sprite.adjustments().0 += STEP;
                }
            }
        }
    }

    fn update_with_time_based_movement(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            if !KEYS_HELD.left && !KEYS_HELD.right {
                return;
            }

            let now = Instant::now();
            let delta_time = if let Some(last_time) = LAST_UPDATE_TIME {
                now.duration_since(last_time).as_secs_f32()
            } else {
                0.0
            };
            LAST_UPDATE_TIME = Some(now);

            let (maxw, _) = board.0.size(ctx);
            
            if let Some(sprite) = board.2.get_mut("player") {
                let current_pos = sprite.position(ctx).0;
                let movement_delta = MOVEMENT_SPEED * delta_time;
                
                if KEYS_HELD.left && current_pos > 5.0 {
                    sprite.adjustments().0 -= movement_delta;
                }
                if KEYS_HELD.right && current_pos < maxw - sprite.dimensions().0 - 5.0 {
                    sprite.adjustments().0 += movement_delta;
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
        let player_info = if let Some(sprite) = board.2.get_mut("player") {
            Some((sprite.position(ctx), *sprite.dimensions()))
        } else {
            None
        };

        if let Some((pos, size)) = player_info {
            Self::shoot(ctx, board, pos, size);
        }
    }

    fn shoot(ctx: &mut Context, board: &mut Gameboard, player_pos: (f32, f32), player_size: (f32, f32)) {
        let now = Instant::now();
        unsafe {
            if let Some(last) = LAST_SHOT_TIME {
                if now.duration_since(last) < SHOOT_COOLDOWN {
                    return;
                }
            }
            LAST_SHOT_TIME = Some(now);
        }

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