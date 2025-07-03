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

mod game;
mod server;
mod collision;
mod settings;
mod player;
mod fly;

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
        
        match GameServer::new() {
            Ok((mut server, receiver)) => {
                if let Err(e) = server.start() {
                    eprintln!("Failed to start game server: {}", e);
                } else {
                    println!("Game server started successfully!");
                }
                
                let event_handler = ServerEventHandler::new(receiver);
                
                Box::new(Galaga::new_with_server(ctx, server, event_handler))
            }
            Err(e) => {
                eprintln!("Failed to initialize game server: {}", e);
                Box::new(Galaga::new(ctx))
            }
        }
    }
}

start!(MyApp);