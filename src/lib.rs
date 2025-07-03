use pelican_ui::{Context, Plugins, Plugin, maverick_start, start, Application, PelicanEngine, MaverickOS, HardwareContext, runtime::Services};
use pelican_ui::drawable::Drawable;
use pelican_ui_std::{AvatarIconStyle, AvatarContent, Interface, NavigateEvent, AppPage};
use pelican_ui::runtime::{Service, ServiceList};
use std::any::TypeId;
use std::pin::Pin;
use std::future::Future;
use pelican_ui::events::{Event, Key, KeyboardEvent, KeyboardState, NamedKey};
use std::collections::BTreeMap;
use std::ptr::addr_of_mut;
use pelican_ui::include_assets;

mod player;
mod game;
mod server;
mod fly_manager;
mod fly_bullets;
mod fly_movement;
mod fly_utils;
mod fly_patterns;
mod fly_state;
mod collision;

use game::Galaga;
use server::{GameServer, ServerEventHandler};

pub struct MyApp;

impl Services for MyApp {
    fn services() -> ServiceList {
        ServiceList::default()
    }
}

impl Plugins for MyApp {
    fn plugins(ctx: &mut Context) -> Vec<Box<dyn Plugin>> {
        vec![]
    }
}

impl Application for MyApp {
    async fn new(ctx: &mut Context) -> Box<dyn Drawable> {
        ctx.assets.include_assets(include_assets!("./assets"));
        
        // Initialize the server and event handler
        match GameServer::new() {
            Ok((mut server, receiver)) => {
                // Start the server on a separate thread
                if let Err(e) = server.start() {
                    eprintln!("Failed to start game server: {}", e);
                } else {
                    println!("Game server started successfully!");
                }
                
                // Create the event handler
                let event_handler = ServerEventHandler::new(receiver);
                
                // Create the game with the server components
                Box::new(Galaga::new_with_server(ctx, server, event_handler))
            }
            Err(e) => {
                eprintln!("Failed to initialize game server: {}", e);
                // Fall back to game without server
                Box::new(Galaga::new(ctx))
            }
        }
    }
}

start!(MyApp);