use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding};

use pelican_game_engine::{CollisionEvent, AspectRatio, Sprite, GameGrid, GameboardBackground};

use std::collections::HashMap;

// use crate::structs::Player;

#[derive(Debug)]
pub struct Galaga;

impl Galaga {
    pub fn new(ctx: &mut Context) -> Gameboard {
        let mut gameboard = Gameboard::new(ctx, AspectRatio::OneOne);
        let player = Sprite::new(ctx, "player", "ship.png", (30.0, 30.0), (Offset::Center, Offset::End));
        gameboard.insert_sprite(ctx, player);

        gameboard
    }
}

#[derive(Debug, Component)]
pub struct Gameboard(GameGrid, GameboardBackground, HashMap<String, Sprite>);

impl Gameboard {
    pub fn new(ctx: &mut Context, aspect_ratio: AspectRatio) -> Self {
        let colors = ctx.theme.colors;
        let background = GameboardBackground::new(ctx, 1.0, 8.0, colors.background.secondary, aspect_ratio);
        Gameboard(GameGrid::new(HashMap::from([("background".to_string(), (Offset::Start, Offset::Start))]), aspect_ratio), background, HashMap::new())
    }

    pub fn insert_sprite(&mut self, ctx: &mut Context, sprite: Sprite) {
        // let (maxw, maxh) = self.0.size(ctx);
        // location.0 = Offset::Static(location.0.get(sprite.dimensions().0, maxw));
        // location.1 = Offset::Static(location.1.get(sprite.dimensions().1, maxh));
        self.0.0.insert(sprite.id().to_string(), *sprite.offset());
        self.2.insert(sprite.id().to_string(), sprite);
    }
}

impl OnEvent for Gameboard {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let (maxw, maxh) = self.0.size(ctx);
            self.2.iter_mut().for_each(|(_, s)| {
                if let Some(location) = self.0.0.get_mut(s.id()) {
                    location.0 = Offset::Static(s.offset().0.get(s.dimensions().0, maxw).abs());
                    location.1 = Offset::Static(s.offset().1.get(s.dimensions().1, maxh).abs());
                }
            });
            // let (maxw, maxh) = self.dimensions();
            // let x = self.0.0.get(max_size.0, maxw);
            // let y = self.0.1.get(max_size.1, maxh);
            // (x, y)
            // let GameboardSize(maxw, maxh) = ctx.state().get::<GameboardSize>();
            // let children = &self.2;

            // children.iter().enumerate().for_each(|(i, a)| {
            //     let (ax, ay) = a.position((maxw, maxh));
            //     let (aw, ah) = a.dimensions();

            //     children.iter().skip(i + 1).for_each(|b| {
            //         let (bx, by) = a.position((maxw, maxh));
            //         let (bw, bh) = b.dimensions();

            //         if  ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by {
            //             ctx.trigger_event(CollisionEvent(a.id()));
            //             ctx.trigger_event(CollisionEvent(b.id()));
            //         }
            //     });
            // });
        } else if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref::<KeyboardEvent>() { 
            match key {
                Key::Named(NamedKey::ArrowLeft) => {
                    let (maxw, maxh) = self.0.size(ctx);
                    let movement_speed = 5.0;
                    if let Some(player) = self.2.get_mut("player") {
                        let offsets = self.0.0.get_mut("player").unwrap();
                        if let Offset::Static(a) = &mut offsets.0 {*a -= movement_speed;}
                    }
                },
                Key::Named(NamedKey::ArrowRight) => {
                    let (maxw, maxh) = self.0.size(ctx);
                    let movement_speed = 5.0;
                    if let Some(player) = self.2.get_mut("player") {
                        let offsets = self.0.0.get_mut("player").unwrap();
                        if let Offset::Static(a) = &mut offsets.0 {*a += movement_speed;}
                    }
                },
                _ => {}
            }
        }
        true
    }
}