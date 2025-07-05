use std::ptr::addr_of_mut;
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use local_ip_address::local_ip;
use pelican_ui::Context;

use crate::settings::GameSettings;

#[derive(Debug, Clone)]
pub enum ServerEvent {
    RightPeak(i32),
    LeftPeak(i32),
    ShootPeak(i32),
    ConnectionEstablished,
    ConnectionLost,
}

#[derive(Debug, Clone)]
pub enum GameAction {
    MoveRight,
    MoveLeft,
    Shoot,
}

pub struct GameServer {
    runtime: tokio::runtime::Runtime,
    event_sender: Sender<ServerEvent>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl GameServer {
    const PORT: u16 = 3030;

    pub fn new() -> Result<(Self, Receiver<ServerEvent>), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let runtime = tokio::runtime::Runtime::new()?;

        let server = GameServer {
            runtime,
            event_sender: tx,
            server_handle: None,
        };

        Ok((server, rx))
    }

    fn get_server_address() -> Result<String, Box<dyn std::error::Error>> {
        let local_ip = local_ip()?;
        Ok(format!("{}:{}", local_ip, Self::PORT))
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let server_address = Self::get_server_address()?;
        println!("WebSocket Game server starting on {}", server_address);
        println!("Local IP address: {}", local_ip()?);
        
        let event_sender = self.event_sender.clone();
        
        let handle = self.runtime.spawn(async move {
            if let Err(e) = Self::run_server(event_sender).await {
                println!("Server error: {}", e);
            }
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }

    async fn run_server(event_sender: Sender<ServerEvent>) -> Result<(), Box<dyn std::error::Error>> {
        let server_address = Self::get_server_address()?;
        let listener = TcpListener::bind(&server_address).await?;
        println!("WebSocket server listening on {}", server_address);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from: {}", addr);
                    let _ = event_sender.send(ServerEvent::ConnectionEstablished);
                    
                    let sender_clone = event_sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, sender_clone).await {
                            println!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(
        stream: TcpStream,
        event_sender: Sender<ServerEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ws_stream = accept_async(stream).await?;
        println!("WebSocket connection established");

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let welcome_msg = Message::Text("Connected to game server".into());
        ws_sender.send(welcome_msg).await?;

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                    
                    match Self::parse_game_message(&text) {
                        Ok((action_type, value)) => {
                            let event = match action_type.as_str() {
                                "right" => Some(ServerEvent::RightPeak(value)),
                                "left" => Some(ServerEvent::LeftPeak(value)),
                                "shoot" => Some(ServerEvent::ShootPeak(value)),
                                _ => None,
                            };

                            match event {
                                Some(server_event) => {
                                    let _ = event_sender.send(server_event);
                                    let response = Message::Text("OK".into());
                                    ws_sender.send(response).await?;
                                }
                                None => {
                                    let response = Message::Text("Unknown action".into());
                                    ws_sender.send(response).await?;
                                }
                            }
                        }
                        Err(e) => {
                            println!("Parse error: {}", e);
                            let response = Message::Text("Parse error".into());
                            ws_sender.send(response).await?;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket connection closed");
                    let _ = event_sender.send(ServerEvent::ConnectionLost);
                    break;
                }
                Ok(Message::Ping(payload)) => {
                    ws_sender.send(Message::Pong(payload)).await?;
                }
                Ok(_) => {
                }
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    let _ = event_sender.send(ServerEvent::ConnectionLost);
                    break;
                }
            }
        }

        Ok(())
    }

    fn parse_game_message(message: &str) -> Result<(String, i32), Box<dyn std::error::Error + Send + Sync>> {
        let json: Value = serde_json::from_str(message)?;

        let action = json.get("action")
            .and_then(|a| a.as_str())
            .ok_or("Action field not found")?;

        let value = json.get("value")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or("Value field not found or not a number")?;

        Ok((action.to_string(), value))
    }
}

impl Drop for GameServer {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct ServerEventHandler {
    receiver: Receiver<ServerEvent>,
}

impl ServerEventHandler {
    pub fn new(receiver: Receiver<ServerEvent>) -> Self {
        Self { receiver }
    }

    pub fn check_events(&self) -> Vec<ServerEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn process_events_for_game(&self, ctx: &mut Context) -> Option<GameAction> {
        let events = self.check_events();

        let peak_min = if let Some(settings) = ctx.state().get_mut::<GameSettings>() {
            settings.get_peak_min()
        } else {
            500.0 
        };

        for event in events {
            match event {
                ServerEvent::RightPeak(value) => {
                    println!("Right peak: {} (min required: {})", value, peak_min);
                    if value as f32 >= peak_min {
                        println!("Right peak exceeds minimum, sending move right action");
                        return Some(GameAction::MoveRight);
                    } else {
                        println!("Right peak below minimum threshold, ignoring");
                    }
                }
                ServerEvent::LeftPeak(value) => {
                    println!("Left peak: {} (min required: {})", value, peak_min);
                    if value as f32 >= peak_min {
                        println!("Left peak exceeds minimum, sending move left action");
                        return Some(GameAction::MoveLeft);
                    } else {
                        println!("Left peak below minimum threshold, ignoring");
                    }
                }
                ServerEvent::ShootPeak(value) => {
                    println!("Shoot peak: {} (min required: {})", value, peak_min);
                    if value as f32 >= peak_min {
                        println!("Shoot peak exceeds minimum, sending shoot action");
                        return Some(GameAction::Shoot);
                    } else {
                        println!("Shoot peak below minimum threshold, ignoring");
                    }
                }
                ServerEvent::ConnectionEstablished => {
                    println!("Connection established");
                }
                ServerEvent::ConnectionLost => {
                    println!("Connection lost");
                }
            }
        }

        None
    }
}