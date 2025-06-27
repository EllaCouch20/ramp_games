use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{CollisionEvent, AspectRatio, Sprite, GameLayout, GameboardBackground, Gameboard, SpriteAction};

use std::collections::HashMap;

// use crate::structs::Player;

const STEP: f32 = 5.0;
const BULLET_SPEED: f32 = 8.0;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));
        let player = Sprite::new(ctx, "player", "spaceship_blue.png", (50.0, 50.0), (Offset::Center, Offset::End));
        gameboard.insert_sprite(ctx, player);
        gameboard
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
                    let mut bullet = Sprite::new(ctx, &bullet_id, "bullet_blue.png", b_size, (Offset::Static(x + ((size.0 - b_size.0) / 2.0)), Offset::Static(y - 20.0)));
                    board.insert_sprite(ctx, bullet);
                },
                _ => {}
            }
        }
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let (maxw, maxh) = board.0.size(ctx);
            
            board.2.iter_mut().for_each(|(id, s)| {
                if let Some(location) = board.0.0.get_mut(s.id()) {
                    let (x, y) = s.position(ctx);
                    location.0 = Offset::Static(x);
                    location.1 = Offset::Static(y);
                }
                
                if id.starts_with("bullet_") {
                    s.adjustments().1 -= BULLET_SPEED; 
                }
                
                // *s.dimensions() = (maxw / 20.0, maxw / 20.0); TODO: Need to keep everything a percentage of screen size or prevent resizing
            });
            
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref::<KeyboardEvent>() {
             
            match key {
                Key::Named(NamedKey::ArrowLeft) => Self::sprite_action(ctx, board, "player", SpriteAction::MoveLeft),
                Key::Named(NamedKey::ArrowRight) => Self::sprite_action(ctx, board, "player", SpriteAction::MoveRight),
                Key::Character(c) if c == "a" => Self::sprite_action(ctx, board, "player", SpriteAction::MoveLeft),
                Key::Character(c) if c == "d" => Self::sprite_action(ctx, board, "player", SpriteAction::MoveRight),
                Key::Named(NamedKey::ArrowUp) => Self::sprite_action(ctx, board, "player", SpriteAction::Shoot),
                Key::Character(c) if c == " " => Self::sprite_action(ctx, board, "player", SpriteAction::Shoot),
                _ => {}
            }
        }
        true
    }
}
// Example: sprite.adjustments().1 is the y adjustment (moving up)
