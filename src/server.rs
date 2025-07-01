use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum ServerEvent {
    RightPeak(i32),
    LeftPeak(i32),
    ShootPeak(i32),
    ConnectionEstablished,
    ConnectionLost,
}

pub struct GameServer {
    event_sender: Sender<ServerEvent>,
    server_thread: Option<thread::JoinHandle<()>>,
}

impl GameServer {
    pub fn new() -> (Self, Receiver<ServerEvent>) {
        let (tx, rx) = mpsc::channel();

        let server = GameServer {
            event_sender: tx,
            server_thread: None,
        };

        (server, rx)
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event_sender = self.event_sender.clone();

        let handle = thread::spawn(move || {
            if let Err(e) = Self::run_server(event_sender) {
                eprintln!("Server error: {}", e);
            }
        });

        self.server_thread = Some(handle);
        println!("Game server started on 192.168.1.110:3030");
        Ok(())
    }

    fn run_server(event_sender: Sender<ServerEvent>) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("192.168.1.110:3030")?;
        println!("Server listening on 192.168.1.110:3030");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let sender_clone = event_sender.clone();
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, sender_clone) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_client(
        mut stream: TcpStream,
        event_sender: Sender<ServerEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received request:\n{}", request);

        let _ = event_sender.send(ServerEvent::ConnectionEstablished);

        match Self::parse_http_request(&request) {
            Some(parsed_request) => {
                let path = parsed_request.path.as_str();
                let method = parsed_request.method.as_str();

                if method == "POST" {
                    let body = parsed_request.body.as_deref().unwrap_or("");
                    match Self::parse_json_body(body) {
                        Ok(peak_value) => {
                            let response = Self::create_response(200, "OK", "Peak received successfully");
                            stream.write_all(response.as_bytes())?;

                            match path {
                                "/peakright" => {
                                    let _ = event_sender.send(ServerEvent::RightPeak(peak_value));
                                }
                                "/leftpeak" => {
                                    let _ = event_sender.send(ServerEvent::LeftPeak(peak_value));
                                }
                                "/shootpeak" => {
                                    let _ = event_sender.send(ServerEvent::ShootPeak(peak_value));
                                }
                                _ => {
                                    let response = Self::create_response(404, "Not Found", "Endpoint not found");
                                    stream.write_all(response.as_bytes())?;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error parsing JSON: {}", e);
                            let response = Self::create_response(400, "Bad Request", "Invalid JSON");
                            stream.write_all(response.as_bytes())?;
                        }
                    }
                } else {
                    let response = Self::create_response(404, "Not Found", "Only POST supported");
                    stream.write_all(response.as_bytes())?;
                }
            }
            None => {
                let response = Self::create_response(400, "Bad Request", "Invalid HTTP request");
                stream.write_all(response.as_bytes())?;
            }
        }

        stream.flush()?;
        Ok(())
    }

    fn parse_json_body(body: &str) -> Result<i32, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(body)?;

        if let Some(peak) = json.get("peak") {
            if let Some(peak_num) = peak.as_i64() {
                Ok(peak_num as i32)
            } else {
                Err("Peak value is not a number".into())
            }
        } else {
            Err("Peak field not found in JSON".into())
        }
    }

    fn create_response(status_code: u16, status_text: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
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

        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() < 2 {
            return None;
        }

        let method = request_line_parts[0].to_string();
        let path = request_line_parts[1].to_string();

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

        let body = if let Some(start_index) = body_start_index {
            if start_index < lines.len() {
                Some(lines[start_index..].join("\n"))
            } else {
                None
            }
        } else {
            None
        };

        Some(HttpRequest {
            method,
            path,
            headers,
            body,
        })
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Drop for GameServer {
    fn drop(&mut self) {
        if let Some(handle) = self.server_thread.take() {
            println!("Shutting down game server...");
            let _ = handle.join();
        }
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

#[derive(Debug, Clone)]
pub enum GameAction {
    MoveRight,
    MoveLeft,
    Shoot,
}
