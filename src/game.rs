use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey, OnEvent, TickEvent};
use pelican_ui::drawable::{Align, Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use pelican_ui_std::{Stack, Content, Header, Bumper, Page, Button, Offset, TextStyle, Text, AppPage, Size, Padding, Column, Wrap, Row, ButtonSize, ButtonWidth, ButtonStyle, ButtonState, IconButton, NavigateEvent, DataItem};
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
const GAME_OVER_RESET_DELAY: Duration = Duration::from_secs(3);
const LIFE_SPRITE_SIZE: (f32, f32) = (20.0, 20.0);
const LIFE_SPRITE_SPACING: f32 = 35.0;
const LIFE_SPRITE_START_X: f32 = 20.0;
const LIFE_SPRITE_Y: f32 = 20.0;

static mut EXPLOSIONS: Option<HashMap<String, Instant>> = None;
static mut ENEMIES_CREATED: bool = false;
static mut PLAYER_RESPAWN_TIME: Option<Instant> = None;
static mut PLAYER_IS_DEAD: bool = false;
static mut SCORE: u32 = 0;
static mut SERVER_EVENT_HANDLER: Option<ServerEventHandler> = None;
static mut GAME_SERVER: Option<GameServer> = None;
static mut GAME_SETTINGS: Option<GameSettings> = None;
static mut GAME_OVER_TIME: Option<Instant> = None;
static mut GAME_IS_OVER: bool = false;

pub struct SettingsButton;
impl SettingsButton {
    pub fn new (
        ctx: &mut Context,
        label: &str,
        description: &str,
        buttons: Vec<(&'static str, &str, Box<dyn FnMut(&mut Context)>)>,
    ) -> Box<dyn Drawable> {
        let buttons = buttons.into_iter().map(|(i, l, c)| {
            Button::secondary(ctx, Some(i), l, None, c)
        }).collect();

        Box::new(DataItem::new(ctx, None, label, None, Some(description), None, Some(buttons)))
    }
}


#[derive(Debug, Component)]
pub struct Settings(Stack, Page);
impl OnEvent for Settings {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AdjustPressureEvent(p)) = event.downcast_ref::<AdjustPressureEvent>() {
            let mut peak = &mut ctx.state().get_mut::<GameSettings>().unwrap().peak_min;
            if  *peak < 1000.0 {
                ctx.state().get_mut::<GameSettings>().unwrap().peak_min += p;

                println!("peak: {}",  ctx.state().get_mut::<GameSettings>().unwrap().peak_min);

                *self.1.content().find_at::<DataItem>(0).unwrap().label() = format!("Touchpad Pressure: {:.0}", ctx.state().get_mut::<GameSettings>().unwrap().peak_min);
            }
        } else if event.downcast_ref::<ToggleFliesShoot>().is_some() {
            let can_shoot = !ctx.state().get_mut::<GameSettings>().unwrap().can_shoot;
            ctx.state().get_mut::<GameSettings>().unwrap().can_shoot = can_shoot;
            let val = if can_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(1).unwrap().label() = format!("Enemy Flies Can Shoot: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(1).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if can_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoMove>().is_some() {
            let player_auto_move = !ctx.state().get_mut::<GameSettings>().unwrap().player_auto_move;
            ctx.state().get_mut::<GameSettings>().unwrap().player_auto_move = player_auto_move;
            let val = if player_auto_move {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(2).unwrap().label() = format!("Player Auto Moves: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(2).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_auto_move { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleAutoShoot>().is_some() {
            let player_auto_shoot = !ctx.state().get_mut::<GameSettings>().unwrap().player_auto_shoot;
            ctx.state().get_mut::<GameSettings>().unwrap().player_auto_shoot = player_auto_shoot;
            let val = if player_auto_shoot {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(3).unwrap().label() = format!("Player Auto Shoots: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(3).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_auto_shoot { "Turn Off".to_string() } else { "Turn On".to_string() };
        } else if event.downcast_ref::<ToggleInvincibility>().is_some() {
            let player_invincible = !ctx.state().get_mut::<GameSettings>().unwrap().player_invincible;
            ctx.state().get_mut::<GameSettings>().unwrap().player_invincible = player_invincible;
            let val = if player_invincible {"Yes"} else {"No"};
            *self.1.content().find_at::<DataItem>(4).unwrap().label() = format!("Player Auto Shoots: {}", val);
            let buttons = &mut self.1.content().find_at::<DataItem>(4).unwrap().buttons();
            let label = &mut buttons.as_mut().unwrap()[0].label().as_mut().unwrap().text().spans[0].text;
            *label = if player_invincible { "Turn Off".to_string() } else { "Turn On".to_string() };
        }
        true
    }
}

impl AppPage for Settings {
    fn has_nav(&self) -> bool {false}
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        match index {
            0 => Ok(Box::new(Galaga::new(ctx))),
            _ => Err(self)
        }
    }
}


impl Settings {
    pub fn new(ctx: &mut Context) -> Self {
        let pressure = format!("Touchpad Pressure: {:.0}", ctx.state().get_mut::<GameSettings>().unwrap().peak_min);
        let can_shoot = format!("Enemy Flies Can Shoot: {}", if ctx.state().get_mut::<GameSettings>().unwrap().can_shoot {"Yes"} else {"No"});
        let auto_move = format!("Player Auto Moves: {}", if ctx.state().get_mut::<GameSettings>().unwrap().player_auto_move {"Yes"} else {"No"});
        let auto_shoot = format!("Player Auto Shoots: {}", if ctx.state().get_mut::<GameSettings>().unwrap().player_auto_shoot {"Yes"} else {"No"});
        let invincible = format!("Player Is Invincible: {}", if ctx.state().get_mut::<GameSettings>().unwrap().player_invincible {"Yes"} else {"No"});

        let buttons = vec![
            SettingsButton::new(ctx, &pressure, "Increase or decrease pressure required to perform an action.", vec![
                ("add", "Decrease", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(-50.0))) as Box<dyn FnMut(&mut Context)>),
                ("add", "Increase", Box::new(|ctx: &mut Context| ctx.trigger_event(AdjustPressureEvent(50.0))) as Box<dyn FnMut(&mut Context)>),
            ]),
            SettingsButton::new(ctx, &can_shoot, "Allows enemy flies to shoot back.", vec![
                ("add", "Turn Off", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleFliesShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            SettingsButton::new(ctx, &auto_move, "Allows player to move back and forth automatically.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoMove)) as Box<dyn FnMut(&mut Context)>)
            ]),
            SettingsButton::new(ctx, &auto_shoot, "Allows player to automatically shoot every 200 millis.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleAutoShoot)) as Box<dyn FnMut(&mut Context)>)
            ]),
            SettingsButton::new(ctx, &invincible, "Allows player to be invincible to enemy fire.", vec![
                ("add", "Turn On", Box::new(|ctx: &mut Context| ctx.trigger_event(ToggleInvincibility)) as Box<dyn FnMut(&mut Context)>)
            ]),
        ];

        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, Some(back), "Settings", None);
        let content = Content::new(Offset::Start, buttons);

        Settings(Stack::default(), Page::new(Some(header), content, None))
    }
}

#[derive(Debug, Component)]
pub struct Galaga(Column, Header, Text, Gameboard);
impl OnEvent for Galaga {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        self.2.text().spans[0].text = format!("Score: {}", unsafe { SCORE });
        true
    }
}

impl AppPage for Galaga {
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> {
        println!("NAVIGATING TO PAGE {}", index);
        match index {
            0 => Ok(Box::new(Settings::new(ctx))),
            _ => Err(self)
        }
    }
    fn has_nav(&self) -> bool {false}
}

impl Galaga {
    pub fn new(ctx: &mut Context) -> Self {
        if ctx.state().get_mut::<GameSettings>().is_none() {Self::initialize_game_state(ctx);}
        let gameboard = Self::create_gameboard(ctx);

        let settings = IconButton::navigation(ctx, "settings", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, None, "Galaga", Some(settings));
        let text_size = ctx.theme.fonts.size.h4;
        let text = Text::new(ctx, "Score: 0", TextStyle::Heading, text_size, Align::Center);
        Galaga(Column::center(24.0), header, text, gameboard)
    }

    pub fn new_with_server(ctx: &mut Context, server: GameServer, event_handler: ServerEventHandler) -> Self {
        if ctx.state().get_mut::<GameSettings>().is_none() {Self::initialize_game_state(ctx);}

        unsafe {
            GAME_SERVER = Some(server);
            SERVER_EVENT_HANDLER = Some(event_handler);
        }

        Self::new(ctx)
    }

    fn initialize_game_state(ctx: &mut Context) {
        unsafe {
            ENEMIES_CREATED = false;
            EXPLOSIONS = Some(HashMap::new());
            PLAYER_RESPAWN_TIME = None;
            PLAYER_IS_DEAD = false;
            SCORE = 0;
            GAME_SETTINGS = Some(GameSettings::new());
            GAME_OVER_TIME = None;
            GAME_IS_OVER = false;
        }

        ctx.state().set(GameSettings::new());

        PlayerLives::initialize_with_lives(4);
        EnemyManager::initialize();
        PlayerManager::initialize();
    }

    fn reset_game_state(ctx: &mut Context, board: &mut Gameboard) {
        println!("RESETTING GAME STATE");

        Self::clear_all_sprites(ctx, board);

        unsafe {
            ENEMIES_CREATED = false;
            EXPLOSIONS = Some(HashMap::new());
            PLAYER_RESPAWN_TIME = None;
            PLAYER_IS_DEAD = false;
            SCORE = 0;
            GAME_OVER_TIME = None;
            GAME_IS_OVER = false;
        }

        PlayerLives::initialize_with_lives(4);
        EnemyManager::initialize();
        PlayerManager::initialize();

        let player = PlayerManager::create_player(ctx);
        board.insert_sprite(ctx, player);

        Self::update_score_display(ctx, board);

        println!("*** GAME RESET COMPLETE ***");
    }

    fn clear_all_sprites(ctx: &mut Context, board: &mut Gameboard) {
        let sprite_ids: Vec<String> = board.2.keys().cloned().collect();
        for sprite_id in sprite_ids {
            Self::remove_sprite_from_board(ctx, board, &sprite_id);
        }
    }

    fn create_gameboard(ctx: &mut Context) -> Gameboard {
        let mut gameboard = Gameboard::new(ctx, AspectRatio::FiveSeven, Box::new(Self::on_event));
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

            Self::create_life_sprites(ctx, board, lives_info.lives);
        }
    }

    fn update_score_display(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let score = SCORE;
            let lives_info = PlayerLives::get_display_info();
            let wave = EnemyManager::get_wave_count();

            Self::update_life_sprites(ctx, board, lives_info.lives);
        }
    }

    fn create_life_sprites(ctx: &mut Context, board: &mut Gameboard, lives: u32) {
        Self::remove_all_life_sprites(ctx, board);

        for i in 0..lives {
            let life_sprite_id = format!("life_{}", i);
            let x_pos = LIFE_SPRITE_START_X + (i as f32 * LIFE_SPRITE_SPACING);

            let mut life_sprite = Sprite::new(
                ctx,
                &life_sprite_id,
                "spaceship",
                LIFE_SPRITE_SIZE,
                (Offset::Static(x_pos), Offset::Static(LIFE_SPRITE_Y))
            );

            life_sprite.adjustments().0 = 0.0;
            life_sprite.adjustments().1 = 0.0;

            board.insert_sprite(ctx, life_sprite);
        }
    }

    fn update_life_sprites(ctx: &mut Context, board: &mut Gameboard, lives: u32) {
        let current_life_sprites: Vec<String> = board.2.keys()
            .filter(|id| id.starts_with("life_"))
            .cloned()
            .collect();

        let current_count = current_life_sprites.len() as u32;

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
            if !GAME_IS_OVER {
                SCORE += points;
                let score = SCORE;
                println!("Score: {}", score);
            }
        }
        Self::update_score_display(ctx, board);
    }

    fn lose_life(ctx: &mut Context, board: &mut Gameboard) {
        PlayerLives::handle_player_death();
        let lives_info = PlayerLives::get_display_info();
        println!("PLAYER died! Lives remaining: {}", lives_info.lives);

        if PlayerLives::is_game_over() {
            unsafe {
                GAME_IS_OVER = true;
                GAME_OVER_TIME = Some(Instant::now());
            }
            println!("GAME OVER! Final Score: {}", unsafe { SCORE });
            Self::remove_all_life_sprites(ctx, board);
        } else {
            Self::update_score_display(ctx, board);
        }
    }

    fn respawn_player(ctx: &mut Context, board: &mut Gameboard) {
        unsafe {
            let can_respawn = !PlayerLives::is_game_over() && !GAME_IS_OVER;

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
            if GAME_IS_OVER {
                return;
            }

            if let Some(ref event_handler) = SERVER_EVENT_HANDLER {
                if let Some(action) = event_handler.process_events_for_game(ctx) {
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
            unsafe {
                if GAME_IS_OVER {
                    if let Some(game_over_time) = GAME_OVER_TIME {
                        if Instant::now().duration_since(game_over_time) >= GAME_OVER_RESET_DELAY {
                            Self::reset_game_state(ctx, board);
                            return true;
                        }
                    }
                    return true;
                }
            }

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
                if !PLAYER_IS_DEAD && !GAME_IS_OVER {
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
                if !PLAYER_IS_DEAD && !GAME_IS_OVER {
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
                if !PLAYER_IS_DEAD && !GAME_IS_OVER {
                    PlayerManager::handle_keyboard_input(ctx, board, keyboard_event);
                }
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct AdjustPressureEvent(pub f32);
impl Event for AdjustPressureEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ToggleFliesShoot;
impl Event for ToggleFliesShoot {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ToggleAutoMove;
impl Event for ToggleAutoMove {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ToggleAutoShoot;
impl Event for ToggleAutoShoot {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

#[derive(Clone, Debug)]
pub struct ToggleInvincibility;
impl Event for ToggleInvincibility {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}