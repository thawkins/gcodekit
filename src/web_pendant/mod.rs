//! Web pendant interface for remote CNC control.
//!
//! This module provides a web-based interface for remote monitoring and control
//! of CNC machines through a web browser, using WebSocket communication.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use warp::Filter;
use warp::ws::{Message, WebSocket};

pub type CommandSender = mpsc::UnboundedSender<WebPendantMessage>;
pub type CommandReceiver = mpsc::UnboundedReceiver<WebPendantMessage>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebPendantMessage {
    StatusUpdate {
        connected: bool,
        position: Option<(f32, f32, f32)>,
        status: String,
    },
    Command {
        command_type: String,
        data: serde_json::Value,
    },
    Response {
        success: bool,
        message: String,
        data: Option<serde_json::Value>,
    },
}

pub struct WebPendant {
    command_sender: mpsc::UnboundedSender<WebPendantMessage>,
    status_receivers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<WebPendantMessage>>>>,
    command_tx: CommandSender,
}

impl Default for WebPendant {
    fn default() -> Self {
        Self::new().0 // Use the WebPendant from the tuple, discard the receiver
    }
}

impl WebPendant {
    pub fn new() -> (Self, CommandReceiver) {
        let (tx, mut rx) = mpsc::unbounded_channel::<WebPendantMessage>();
        let (command_tx, command_rx) = mpsc::unbounded_channel::<WebPendantMessage>();
        let status_receivers = Arc::new(Mutex::new(HashMap::<
            String,
            mpsc::UnboundedSender<WebPendantMessage>,
        >::new()));
        let status_receivers_clone = status_receivers.clone();

        // Spawn a task to handle incoming commands
        let command_tx_clone = command_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    WebPendantMessage::Command { .. } => {
                        // Send command to main app
                        let _ = command_tx_clone.send(msg.clone());

                        // Broadcast status updates to all connected clients
                        let mut receivers = status_receivers_clone.lock().unwrap();
                        receivers.retain(|_, sender| sender.send(msg.clone()).is_ok());
                    }
                    WebPendantMessage::StatusUpdate { .. } => {
                        // Broadcast status updates to all connected clients
                        let mut receivers = status_receivers_clone.lock().unwrap();
                        receivers.retain(|_, sender| sender.send(msg.clone()).is_ok());
                    }
                    WebPendantMessage::Response { .. } => {
                        // Broadcast responses to all connected clients
                        let mut receivers = status_receivers_clone.lock().unwrap();
                        receivers.retain(|_, sender| sender.send(msg.clone()).is_ok());
                    }
                }
            }
        });

        (
            Self {
                command_sender: tx,
                status_receivers,
                command_tx,
            },
            command_rx,
        )
    }

    pub fn send_status_update(
        &self,
        connected: bool,
        position: Option<(f32, f32, f32)>,
        status: String,
    ) {
        let message = WebPendantMessage::StatusUpdate {
            connected,
            position,
            status,
        };

        // Send to command channel for broadcasting to clients
        let _ = self.command_tx.send(message);
    }

    pub async fn start_server(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let status_receivers = self.status_receivers.clone();
        let command_sender = self.command_sender.clone();

        // WebSocket route
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let status_receivers = status_receivers.clone();
                let command_sender = command_sender.clone();
                ws.on_upgrade(move |socket| {
                    Self::handle_websocket(socket, status_receivers, command_sender)
                })
            });

        // Main HTML page
        let index = warp::path::end().map(|| warp::reply::html(Self::get_html_page()));

        let routes = index.or(ws_route).with(warp::cors().allow_any_origin());

        println!("Web pendant server starting on port {}", port);
        warp::serve(routes).run(([127, 0, 0, 1], port)).await;

        Ok(())
    }

    async fn handle_websocket(
        ws: WebSocket,
        status_receivers: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<WebPendantMessage>>>>,
        command_sender: mpsc::UnboundedSender<WebPendantMessage>,
    ) {
        let (mut ws_sender, mut ws_receiver) = ws.split();

        let client_id = uuid::Uuid::new_v4().to_string();
        let (client_tx, mut client_rx) = mpsc::unbounded_channel();

        // Add this client to the receivers
        {
            let mut receivers = status_receivers.lock().unwrap();
            receivers.insert(client_id.clone(), client_tx);
        }

        // Send initial status
        let initial_status = WebPendantMessage::StatusUpdate {
            connected: false,
            position: None,
            status: "Disconnected".to_string(),
        };

        if let Ok(json) = serde_json::to_string(&initial_status) {
            let _ = ws_sender.send(Message::text(json)).await;
        }

        // Handle messages from client
        let client_id_clone = client_id.clone();
        let status_receivers_clone = status_receivers.clone();

        tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_str() {
                            if let Ok(command) = serde_json::from_str::<WebPendantMessage>(text) {
                                let _ = command_sender.send(command);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // Remove client when connection closes
            let mut receivers = status_receivers_clone.lock().unwrap();
            receivers.remove(&client_id_clone);
        });

        // Forward status updates to client
        while let Some(msg) = client_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_sender.send(Message::text(json)).await.is_err() {
                    break;
                }
            }
        }
    }

    fn get_html_page() -> String {
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>gcodekit Web Pendant</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        .status-panel {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .control-panel {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
        }
        .control-group {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .jog-controls {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 10px;
            margin: 20px 0;
        }
        .jog-btn {
            padding: 15px;
            border: none;
            border-radius: 4px;
            background: #007bff;
            color: white;
            font-size: 16px;
            cursor: pointer;
        }
        .jog-btn:hover {
            background: #0056b3;
        }
        .jog-btn.center {
            background: #6c757d;
        }
        .jog-btn.center:hover {
            background: #545b62;
        }
        .status-indicator {
            display: inline-block;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }
        .status-connected {
            background-color: #28a745;
        }
        .status-disconnected {
            background-color: #dc3545;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>gcodekit Web Pendant</h1>
            <p>Remote CNC Machine Control</p>
        </div>

        <div class="status-panel">
            <h2>Machine Status</h2>
            <div class="status-indicator status-disconnected" id="statusIndicator"></div>
            <span id="statusText">Disconnected</span>
            <div id="positionDisplay">X: 0.000 Y: 0.000 Z: 0.000</div>
        </div>

        <div class="control-panel">
            <div class="control-group">
                <h3>Jog Controls</h3>
                <div class="jog-controls">
                    <div></div>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'Y', distance: 1})">Y+</button>
                    <div></div>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'X', distance: -1})">X-</button>
                    <button class="jog-btn center" onclick="sendCommand('home', {})">HOME</button>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'X', distance: 1})">X+</button>
                    <div></div>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'Y', distance: -1})">Y-</button>
                    <div></div>
                </div>
                <div class="jog-controls">
                    <div></div>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'Z', distance: 1})">Z+</button>
                    <div></div>
                    <div></div>
                    <div></div>
                    <div></div>
                    <div></div>
                    <button class="jog-btn" onclick="sendCommand('jog', {axis: 'Z', distance: -1})">Z-</button>
                    <div></div>
                </div>
            </div>

            <div class="control-group">
                <h3>Machine Controls</h3>
                <button class="jog-btn" onclick="sendCommand('emergency_stop', {})" style="background: #dc3545; margin: 10px 0;">EMERGENCY STOP</button>
                <button class="jog-btn" onclick="sendCommand('reset', {})" style="background: #ffc107; color: black; margin: 10px 0;">RESET</button>
            </div>
        </div>
    </div>

    <script>
        let ws = null;
        let reconnectInterval = null;

        function connectWebSocket() {
            ws = new WebSocket('ws://localhost:8080/ws');

            ws.onopen = function(event) {
                console.log('Connected to WebSocket');
                document.getElementById('statusIndicator').className = 'status-indicator status-connected';
                document.getElementById('statusText').textContent = 'Connected';
                clearInterval(reconnectInterval);
            };

            ws.onmessage = function(event) {
                try {
                    const message = JSON.parse(event.data);
                    handleMessage(message);
                } catch (e) {
                    console.error('Failed to parse message:', e);
                }
            };

            ws.onclose = function(event) {
                console.log('WebSocket connection closed');
                document.getElementById('statusIndicator').className = 'status-indicator status-disconnected';
                document.getElementById('statusText').textContent = 'Disconnected';

                // Attempt to reconnect
                reconnectInterval = setInterval(() => {
                    console.log('Attempting to reconnect...');
                    connectWebSocket();
                }, 5000);
            };

            ws.onerror = function(error) {
                console.error('WebSocket error:', error);
            };
        }

        function handleMessage(message) {
            switch(message.type) {
                case 'StatusUpdate':
                    updateStatus(message.connected, message.position, message.status);
                    break;
                case 'Response':
                    handleResponse(message);
                    break;
                default:
                    console.log('Unknown message type:', message.type);
            }
        }

        function updateStatus(connected, position, status) {
            const indicator = document.getElementById('statusIndicator');
            const statusText = document.getElementById('statusText');
            const positionDisplay = document.getElementById('positionDisplay');

            if (connected) {
                indicator.className = 'status-indicator status-connected';
                statusText.textContent = status || 'Connected';
            } else {
                indicator.className = 'status-indicator status-disconnected';
                statusText.textContent = status || 'Disconnected';
            }

            if (position) {
                positionDisplay.textContent = `X:${position[0].toFixed(3)} Y:${position[1].toFixed(3)} Z:${position[2].toFixed(3)}`;
            } else {
                positionDisplay.textContent = 'X: 0.000 Y: 0.000 Z: 0.000';
            }
        }

        function handleResponse(response) {
            if (response.success) {
                console.log('Command executed successfully:', response.message);
            } else {
                console.error('Command failed:', response.message);
                alert('Command failed: ' + response.message);
            }
        }

        function sendCommand(commandType, data) {
            if (ws && ws.readyState === WebSocket.OPEN) {
                const message = {
                    type: 'Command',
                    command_type: commandType,
                    data: data
                };
                ws.send(JSON.stringify(message));
                console.log('Sent command:', message);
            } else {
                alert('Not connected to server. Please wait for connection to be established.');
            }
        }

        // Connect when page loads
        window.onload = function() {
            connectWebSocket();
        };
    </script>
</body>
</html>
        "#.to_string()
    }
}
