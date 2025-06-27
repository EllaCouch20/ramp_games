use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{CollisionEvent, AspectRatio, Sprite, GameGrid, GameboardBackground, Gameboard};

use std::collections::HashMap;

// use crate::structs::Player;

const STEP: f32 = 5.0;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne, Box::new(Self::on_event));
        let player = Sprite::new(ctx, "player", "ship.png", (30.0, 30.0), (Offset::Center, Offset::End));
        gameboard.insert_sprite(ctx, player);

        gameboard
    }

    fn on_event(board: &mut Gameboard, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let (maxw, maxh) = board.0.size(ctx);
            board.2.iter_mut().for_each(|(_, s)| {
                if let Some(location) = board.0.0.get_mut(s.id()) {
                    let sd = s.dimensions().clone();
                    location.0 = Offset::Static(s.offset().0.get(sd.0, maxw).abs() + s.position().0);
                    location.1 = Offset::Static(s.offset().1.get(sd.1, maxh).abs() + s.position().1);
                }
                *s.dimensions() = (maxw / 20.0, maxw / 20.0);
            });
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref::<KeyboardEvent>() { 
            match key {
                Key::Named(NamedKey::ArrowLeft) => {
                    let (maxw, maxh) = board.0.size(ctx);
                    if let Some(player) = board.2.get_mut("player") {
                        let pw = player.dimensions().0;
                        if player.offset().0.get(pw, maxw).abs() + player.position().0 > 5.0 {
                            player.position().0 -= STEP;
                        }
                    }
                },
                Key::Named(NamedKey::ArrowRight) => {
                    let (maxw, maxh) = board.0.size(ctx);
                    if let Some(player) = board.2.get_mut("player") {
                        let pw = player.dimensions().0;
                        if player.offset().0.get(pw, maxw).abs() + player.position().0 < maxw - pw - 5.0 {
                            player.position().0 += STEP;
                        }
                    }
                },
                Key::Named(NamedKey::ArrowUp) => {
                    if let Some(player) = board.2.get_mut("player") {
                        let player_pos = player.position();
                        println!("Player position: {:?}", player_pos);
                        let id = uuid::Uuid::new_v4().to_string();
                        let mut bullet = Sprite::new(ctx, &id, "Bullet_upward.png", (10.0, 20.0), (Offset::Static(player_pos.0), Offset::Static(player_pos.1 - 5.0)));
                        board.insert_sprite(ctx, bullet);
                    }
                },

                _ => {}
            }
        }
        true
    }
}
