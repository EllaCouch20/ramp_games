
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{CollisionEvent, AspectRatio, Sprite, GameLayout, GameboardBackground, Gameboard, SpriteAction};

use std::collections::HashMap;

const STEP: f32 = 5.0;
const BULLET_SPEED: f32 = 8.0;

static mut ENEMIES_CREATED: bool = false;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        unsafe { ENEMIES_CREATED = false; }
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));
        let player = Sprite::new(ctx, "player", "spaceship_blue.png", (50.0, 50.0), (Offset::Center, Offset::End));
        gameboard.insert_sprite(ctx, player);
        
        gameboard
    }

    fn create_enemies(ctx: &mut Context, board: &mut Gameboard) {
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

        for (id, image, x, y) in enemies {
            let sprite = Sprite::new(
                ctx, 
                id, 
                image, 
                (50.0, 50.0), 
                (Offset::Static(x), Offset::Static(y))
            );
            board.insert_sprite(ctx, sprite);
        }
    }

    fn check_collision(sprite1_pos: (f32, f32), sprite1_size: (f32, f32), sprite2_pos: (f32, f32), sprite2_size: (f32, f32)) -> bool {
        let (x1, y1) = sprite1_pos;
        let (w1, h1) = sprite1_size;
        let (x2, y2) = sprite2_pos;
        let (w2, h2) = sprite2_size;
        
        let collision = x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
        
        if collision {
            println!("COLLISION DETECTED!");
            println!("  Bullet: pos=({:.1}, {:.1}), size=({:.1}, {:.1})", x1, y1, w1, h1);
            println!("  Enemy:  pos=({:.1}, {:.1}), size=({:.1}, {:.1})", x2, y2, w2, h2);
        }
        
        collision
    }

    fn remove_sprite_from_board(ctx: &mut Context, board: &mut Gameboard, sprite_id: &str) {
        if board.2.remove(sprite_id).is_some() {
            board.0.0.remove(sprite_id);
            println!("Removed sprite: {}", sprite_id);
        } else {
            println!("Warning: Tried to remove non-existent sprite: {}", sprite_id);
        }
    }
    
    fn sprite_action(ctx: &mut Context, board: &mut Gameboard, name: &str, action: SpriteAction) {
        let (maxw, maxh) = board.0.size(ctx);
        if let Some(sprite) = board.2.get_mut(name) {
            match action {
                SpriteAction::MoveLeft => {
                    if sprite.position(ctx).0 > 5.0 { sprite.adjustments().0 -= STEP; }
                },
                SpriteAction::MoveRight => {
                    if sprite.position(ctx).0 < maxw - sprite.dimensions().0 - 5.0 { sprite.adjustments().0 += STEP; }
                },
                SpriteAction::Shoot => {
                    let b_size = (15.0, 15.0);
                    let size = sprite.dimensions().clone();
                    let (x, y) = sprite.position(ctx);
                    let bullet_id = format!("bullet_{}", uuid::Uuid::new_v4());
                    let bullet = Sprite::new(ctx, &bullet_id, "bullet_blue.png", b_size, (Offset::Static(x + ((size.0 - b_size.0) / 2.0)), Offset::Static(y - 20.0)));
                    board.insert_sprite(ctx, bullet);
                },
                _ => {}
            }
        }
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
    if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
        unsafe {
            if !ENEMIES_CREATED {
                Self::create_enemies(ctx, board);
                ENEMIES_CREATED = true;
            }
        }
        
        let (_maxw, _maxh) = board.0.size(ctx);
        
        let mut bullets_to_remove = Vec::new();
        let mut active_bullets = Vec::new();
        
        for (id, sprite) in board.2.iter_mut() {
            if id.starts_with("bullet_") {
                sprite.adjustments().1 -= BULLET_SPEED;
                
                let pos = sprite.position(ctx);
                
                if pos.1 < -50.0 {
                    bullets_to_remove.push(id.clone());
                } else {
                    let size = *sprite.dimensions();
                    active_bullets.push((id.clone(), pos, size));
                }
            }
        }
        
        for bullet_id in &bullets_to_remove {
            Self::remove_sprite_from_board(ctx, board, bullet_id);
        }
        
        let mut sprites_to_remove = Vec::new();
        
        for (bullet_id, bullet_pos, bullet_size) in active_bullets {
            for (enemy_id, enemy_sprite) in board.2.iter_mut() {
                if enemy_id != "player" && !enemy_id.starts_with("bullet_") {
                    let enemy_pos = enemy_sprite.position(ctx);
                    let enemy_size = *enemy_sprite.dimensions();
                    
                    if Self::check_collision(bullet_pos, bullet_size, enemy_pos, enemy_size) {
                        sprites_to_remove.push(bullet_id.clone());
                        sprites_to_remove.push(enemy_id.clone());
                        break;
                    }
                }
            }
        }
        
        for sprite_id in sprites_to_remove {
            Self::remove_sprite_from_board(ctx, board, &sprite_id);
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
    } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref::<KeyboardEvent>() {
             
            match key {
                Key::Named(NamedKey::ArrowLeft) => Self::sprite_action(ctx, board, "player", SpriteAction::MoveLeft),
                Key::Named(NamedKey::ArrowRight) => Self::sprite_action(ctx, board, "player", SpriteAction::MoveRight),
                Key::Named(NamedKey::ArrowUp) => Self::sprite_action(ctx, board, "player", SpriteAction::Shoot),
                _ => {}
            }
        }
        true
    }
}