use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use serde_json::Value;
use std::net::SocketAddr;

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

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

pub struct GameServer {
    runtime: tokio::runtime::Runtime,
    event_sender: Sender<ServerEvent>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl GameServer {
    const SERVER_ADDRESS: &'static str = "192.168.1.110:3030";

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

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_sender = self.event_sender.clone();
        
        let handle = self.runtime.spawn(async move {
            if let Err(e) = Self::run_server(event_sender).await {
                eprintln!("Server error: {}", e);
            }
        });

        self.server_handle = Some(handle);
        println!("Game server started on {}", Self::SERVER_ADDRESS);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            println!("Game server stopped");
        }
    }

    async fn run_server(event_sender: Sender<ServerEvent>) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(Self::SERVER_ADDRESS).await?;
        println!("Server listening on {}", Self::SERVER_ADDRESS);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    println!("New connection from: {}", addr);
                    let _ = event_sender.send(ServerEvent::ConnectionEstablished);
                    
                    let sender_clone = event_sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, sender_clone).await {
                            eprintln!("Client handling error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(
        mut stream: TcpStream,
        event_sender: Sender<ServerEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0; 1024];
        
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received request from {}: {} bytes", 
                stream.peer_addr()?, bytes_read);

        match Self::parse_http_request(&request) {
            Some(parsed_request) => {
                Self::handle_http_request(parsed_request, &mut stream, event_sender).await?;
            }
            None => {
                let response = Self::create_response(400, "Bad Request", "Invalid HTTP request");
                stream.write_all(response.as_bytes()).await?;
            }
        }

        stream.flush().await?;
        Ok(())
    }

    async fn handle_http_request(
        request: HttpRequest,
        stream: &mut TcpStream,
        event_sender: Sender<ServerEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match request.method.as_str() {
            "POST" => {
                Self::handle_post_request(request, stream, event_sender).await?;
            }
            "OPTIONS" => {
                // Handle CORS preflight
                let response = Self::create_response(200, "OK", "");
                stream.write_all(response.as_bytes()).await?;
            }
            _ => {
                let response = Self::create_response(405, "Method Not Allowed", "Only POST and OPTIONS supported");
                stream.write_all(response.as_bytes()).await?;
            }
        }
        Ok(())
    }

    async fn handle_post_request(
        request: HttpRequest,
        stream: &mut TcpStream,
        event_sender: Sender<ServerEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let body = request.body.as_deref().unwrap_or("");
        
        match Self::parse_json_body(body) {
            Ok(peak_value) => {
                let event = match request.path.as_str() {
                    "/peakright" => Some(ServerEvent::RightPeak(peak_value)),
                    "/leftpeak" => Some(ServerEvent::LeftPeak(peak_value)),
                    "/shootpeak" => Some(ServerEvent::ShootPeak(peak_value)),
                    _ => None,
                };

                match event {
                    Some(server_event) => {
                        let _ = event_sender.send(server_event);
                        let response = Self::create_response(200, "OK", "Peak received successfully");
                        stream.write_all(response.as_bytes()).await?;
                    }
                    None => {
                        let response = Self::create_response(404, "Not Found", "Endpoint not found");
                        stream.write_all(response.as_bytes()).await?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                let response = Self::create_response(400, "Bad Request", "Invalid JSON");
                stream.write_all(response.as_bytes()).await?;
            }
        }
        Ok(())
    }

    fn parse_json_body(body: &str) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let json: Value = serde_json::from_str(body)?;

        json.get("peak")
            .and_then(|peak| peak.as_i64())
            .map(|peak_num| peak_num as i32)
            .ok_or_else(|| "Peak field not found or not a number".into())
    }

    fn create_response(status_code: u16, status_text: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             \r\n\
             {}",
            status_code, status_text, body.len(), body
        )
    }

    fn parse_http_request(request: &str) -> Option<HttpRequest> {
        let lines: Vec<&str> = request.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // Parse request line
        let request_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_parts.len() < 2 {
            return None;
        }

        let method = request_parts[0].to_string();
        let path = request_parts[1].to_string();

        // Parse headers
        let mut headers = HashMap::new();
        let mut body_start_index = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start_index = Some(i + 1);
                break;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // Parse body
        let body = body_start_index
            .filter(|&start_index| start_index < lines.len())
            .map(|start_index| lines[start_index..].join("\n"));

        Some(HttpRequest {
            method,
            path,
            headers,
            body,
        })
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

    pub fn process_events_for_game(&self) -> Option<GameAction> {
        let events = self.check_events();

        for event in events {
            match event {
                ServerEvent::RightPeak(value) => {
                    println!("🎮 Arduino right peak: {} - MOVE RIGHT", value);
                    return Some(GameAction::MoveRight);
                }
                ServerEvent::LeftPeak(value) => {
                    println!("🎮 Arduino left peak: {} - MOVE LEFT", value);
                    return Some(GameAction::MoveLeft);
                }
                ServerEvent::ShootPeak(value) => {
                    println!("🎮 Arduino shoot peak: {} - SHOOT", value);
                    return Some(GameAction::Shoot);
                }
                ServerEvent::ConnectionEstablished => {
                    println!("Arduino connected to game server!");
                }
                ServerEvent::ConnectionLost => {
                    println!("Arduino connection lost!");
                }
            }
        }

        None
    }
}