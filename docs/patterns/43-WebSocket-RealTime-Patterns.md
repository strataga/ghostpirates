# Pattern 43: WebSocket & Real-Time Communication Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Status**: Active

---

## Table of Contents

1. [Overview](#overview)
2. [WebSocket Fundamentals](#websocket-fundamentals)
3. [Native WebSocket with Axum & Tokio](#native-websocket-with-axum--tokio)
4. [Event-Driven Architecture](#event-driven-architecture)
5. [Room & Namespace Management](#room--namespace-management)
6. [Authentication & Authorization](#authentication--authorization)
7. [Message Patterns](#message-patterns)
8. [Scaling WebSockets](#scaling-websockets)
9. [Error Handling & Reconnection](#error-handling--reconnection)
10. [Performance Optimization](#performance-optimization)
11. [Testing WebSockets](#testing-websockets)

---

## Overview

Real-time communication enables instant bidirectional data flow between clients and servers, essential for collaborative features, live updates, and interactive experiences.

### When to Use WebSockets

**✅ Use WebSockets For**:

- Real-time collaboration (time tracking, project updates)
- Live notifications and alerts
- Chat and messaging systems
- Live dashboards and analytics
- Multiplayer features
- Real-time data streaming

**❌ Use REST/Polling For**:

- Infrequent updates (< 1 per minute)
- Request-response patterns
- File uploads/downloads
- Simple CRUD operations
- Public API endpoints

### WebSocket vs Alternatives

| Technology             | Latency | Complexity | Bidirectional        | Browser Support |
| ---------------------- | ------- | ---------- | -------------------- | --------------- |
| **WebSocket**          | Low     | Medium     | ✅                   | Modern browsers |
| **Server-Sent Events** | Low     | Low        | ❌ (server → client) | Modern browsers |
| **Long Polling**       | Medium  | Low        | ❌                   | All browsers    |
| **Short Polling**      | High    | Low        | ❌                   | All browsers    |

---

## WebSocket Fundamentals

### 1. WebSocket Lifecycle

```
Client                          Server
  |                                |
  |---- HTTP Upgrade Request ---→ |
  |←--- 101 Switching Protocols -- |
  |                                |
  |←------ WebSocket Open ------→ |
  |                                |
  |←----- Bidirectional Data ---→ |
  |                                |
  |←-------- WebSocket Close ---→ |
```

### 2. Native WebSocket (Client)

```typescript
// Client-side vanilla WebSocket
const ws = new WebSocket('ws://localhost:3001');

ws.onopen = () => {
  console.log('Connected to server');
  ws.send(JSON.stringify({ type: 'subscribe', room: 'time-entries' }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected from server');
};
```

### 3. Native WebSocket (Server - Rust with Axum)

```rust
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    state: Arc<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
    let client_id = uuid::Uuid::new_v4();

    println!("Client connected: {}", client_id);

    loop {
        tokio::select! {
            // Receive from client
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    // Broadcast to all clients
                    let _ = state.tx.send(text);
                }
            }
            // Broadcast to client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    }

    println!("Client disconnected: {}", client_id);
}

// Router setup
pub fn create_router() -> Router {
    let (tx, _rx) = broadcast::channel(100);
    let state = Arc::new(AppState { tx });

    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state)
}
```

---

## Native WebSocket with Axum & Tokio

### 1. Server Setup (Rust with Message Handling)

```rust
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
struct AppState {
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    tx: broadcast::Sender<ServerMessage>,
}

#[derive(Clone)]
struct ClientInfo {
    id: String,
    user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Subscribe { room: String },
    Message { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    Connected { client_id: String },
    Message { content: String },
    Error { message: String },
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    state: Arc<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let client_id = uuid::Uuid::new_v4().to_string();
    let mut rx = state.tx.subscribe();

    // Add client to state
    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id.clone(), ClientInfo {
            id: client_id.clone(),
            user_id: None,
        });
        println!("Client connected: {}", client_id);
        println!("Total clients: {}", clients.len());
    }

    // Send connection confirmation
    let connected_msg = ServerMessage::Connected {
        client_id: client_id.clone(),
    };
    let _ = socket.send(Message::Text(
        serde_json::to_string(&connected_msg).unwrap()
    )).await;

    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Message { content }) => {
                            println!("Received message: {}", content);
                            let response = ServerMessage::Message { content };
                            let _ = state.tx.send(response);
                        }
                        Ok(ClientMessage::Subscribe { room }) => {
                            println!("Client {} subscribed to room: {}", client_id, room);
                        }
                        Err(e) => {
                            let error_msg = ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                            };
                            let _ = socket.send(Message::Text(
                                serde_json::to_string(&error_msg).unwrap()
                            )).await;
                        }
                    }
                }
            }
            Ok(msg) = rx.recv() => {
                let text = serde_json::to_string(&msg).unwrap();
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    }

    // Remove client on disconnect
    {
        let mut clients = state.clients.write().await;
        clients.remove(&client_id);
        println!("Client disconnected: {}", client_id);
    }
}

pub fn create_router() -> Router {
    let (tx, _rx) = broadcast::channel(100);
    let state = Arc::new(AppState {
        clients: Arc::new(RwLock::new(HashMap::new())),
        tx,
    });

    Router::new()
        .route("/events", get(websocket_handler))
        .with_state(state)
}
```

### 2. Client Setup (React with Native WebSocket)

```typescript
// hooks/useSocket.ts
import { useEffect, useRef, useState } from 'react';

interface UseSocketOptions {
  url: string;
  token?: string;
  autoReconnect?: boolean;
}

export function useSocket({ url, token, autoReconnect = true }: UseSocketOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    let reconnectAttempts = 0;
    const maxReconnectAttempts = 5;

    const connect = () => {
      const wsUrl = token ? `${url}?token=${token}` : url;
      const socket = new WebSocket(wsUrl);

      socket.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        reconnectAttempts = 0;
      };

      socket.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);

        // Auto-reconnect
        if (autoReconnect && reconnectAttempts < maxReconnectAttempts) {
          reconnectAttempts++;
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 5000);
          reconnectTimeoutRef.current = setTimeout(connect, delay);
        }
      };

      socket.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      socketRef.current = socket;
    };

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (socketRef.current) {
        socketRef.current.close();
      }
    };
  }, [url, token, autoReconnect]);

  return {
    socket: socketRef.current,
    isConnected,
  };
}
```

**Usage**:

```typescript
function TimeTrackingDashboard() {
  const { socket, isConnected } = useSocket({
    url: 'ws://localhost:3001/time-tracking',
    token: localStorage.getItem('accessToken') || '',
  });

  useEffect(() => {
    if (!socket) return;

    socket.onmessage = (event) => {
      const data = JSON.parse(event.data);

      switch (data.type) {
        case 'TimeEntryCreated':
          console.log('New time entry:', data);
          // Update UI
          break;
        case 'TimeEntryUpdated':
          console.log('Time entry updated:', data);
          // Update UI
          break;
      }
    };

    // Send subscription message
    const subscribeMsg = JSON.stringify({
      type: 'subscribe',
      room: 'time-tracking',
    });
    socket.send(subscribeMsg);

    return () => {
      // Cleanup handled by hook
    };
  }, [socket]);

  return (
    <div>
      <StatusIndicator connected={isConnected} />
      {/* Dashboard content */}
    </div>
  );
}
```

---

## Event-Driven Architecture

### 1. Event Broadcasting

**Server-Side Event Handler**:

```rust
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeEntryCreated {
    id: String,
    user_id: String,
    project_id: String,
    hours: f64,
    date: String,
    created_at: String,
    organization_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum BroadcastEvent {
    TimeEntryCreated(TimeEntryCreated),
}

struct EventHandler {
    broadcast_tx: broadcast::Sender<BroadcastEvent>,
    rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>, // room_id -> client_ids
}

impl EventHandler {
    async fn handle_time_entry_created(&self, event: TimeEntryCreated) {
        let room_id = format!("organization:{}", event.organization_id);

        // Broadcast to all clients in the organization room
        let broadcast_event = BroadcastEvent::TimeEntryCreated(event);
        let _ = self.broadcast_tx.send(broadcast_event);

        println!("Broadcasted time entry created to room: {}", room_id);
    }
}
```

### 2. Event Subscription Pattern

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Subscribe { room: String },
    Unsubscribe { room: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ServerResponse {
    Subscribed { room: String },
    Unsubscribed { room: String },
    Error { message: String },
}

struct RoomManager {
    rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>, // room_id -> client_ids
    client_rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>, // client_id -> room_ids
}

impl RoomManager {
    fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            client_rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn subscribe(&self, client_id: &str, room: &str, user_org_id: &str) -> Result<(), String> {
        // Validate user has access to this room
        if room.starts_with("organization:") {
            let org_id = room.split(':').nth(1).unwrap_or("");
            if org_id != user_org_id {
                return Err("Access denied to this room".to_string());
            }
        }

        // Add client to room
        {
            let mut rooms = self.rooms.write().await;
            rooms.entry(room.to_string())
                .or_insert_with(HashSet::new)
                .insert(client_id.to_string());
        }

        // Track room for client
        {
            let mut client_rooms = self.client_rooms.write().await;
            client_rooms.entry(client_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(room.to_string());
        }

        Ok(())
    }

    async fn unsubscribe(&self, client_id: &str, room: &str) {
        {
            let mut rooms = self.rooms.write().await;
            if let Some(clients) = rooms.get_mut(room) {
                clients.remove(client_id);
            }
        }

        {
            let mut client_rooms = self.client_rooms.write().await;
            if let Some(rooms) = client_rooms.get_mut(client_id) {
                rooms.remove(room);
            }
        }
    }

    async fn cleanup_client(&self, client_id: &str) {
        // Remove client from all rooms
        let rooms_to_clean = {
            let client_rooms = self.client_rooms.read().await;
            client_rooms.get(client_id).cloned()
        };

        if let Some(room_set) = rooms_to_clean {
            for room in room_set {
                self.unsubscribe(client_id, &room).await;
            }
        }
    }
}
```

**Client**:

```typescript
useEffect(() => {
  if (!socket) return;

  // Subscribe to organization events
  socket.emit('subscribe', { room: `organization:${organizationId}` });

  // Subscribe to user-specific events
  socket.emit('subscribe', { room: `user:${userId}` });

  return () => {
    socket.emit('unsubscribe', { room: `organization:${organizationId}` });
    socket.emit('unsubscribe', { room: `user:${userId}` });
  };
}, [socket, organizationId, userId]);
```

---

## Room & Namespace Management

### 1. Namespaces for Feature Separation (Rust Routing)

```rust
use axum::{
    extract::{Path, State, ws::WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};

// Time tracking messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TimeTrackingMessage {
    StartTimer { project_id: String },
    StopTimer { project_id: String },
    TimerStarted { project_id: String, started_at: String },
}

// Notification messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum NotificationMessage {
    MarkRead { notification_id: String },
    NewNotification { id: String, title: String, body: String },
}

async fn time_tracking_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_time_tracking_socket(socket, state))
}

async fn handle_time_tracking_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let client_id = uuid::Uuid::new_v4().to_string();

    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<TimeTrackingMessage>(&text) {
                        Ok(TimeTrackingMessage::StartTimer { project_id }) => {
                            let user_id = "extracted_from_jwt"; // From auth middleware

                            // Start timer logic here
                            let response = TimeTrackingMessage::TimerStarted {
                                project_id: project_id.clone(),
                                started_at: chrono::Utc::now().to_rfc3339(),
                            };

                            // Send to user's room
                            let msg_text = serde_json::to_string(&response).unwrap();
                            let _ = socket.send(Message::Text(msg_text)).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

async fn notifications_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_notifications_socket(socket, state))
}

async fn handle_notifications_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Handle notification-specific logic
    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<NotificationMessage>(&text) {
                        Ok(NotificationMessage::MarkRead { notification_id }) => {
                            // Mark notification as read
                            println!("Marking notification {} as read", notification_id);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// Create router with namespace-like routing
pub fn create_feature_router() -> Router {
    let state = Arc::new(AppState::new());

    Router::new()
        .route("/time-tracking", get(time_tracking_handler))
        .route("/notifications", get(notifications_handler))
        .with_state(state)
}
```

### 2. Room Patterns

**Organization Rooms**:

```rust
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

struct ConnectionManager {
    organization_rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    user_rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    client_senders: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
}

impl ConnectionManager {
    // Join organization room on connection
    async fn handle_connection(&self, client_id: String, user_org_id: String, user_id: String) {
        let room_id = format!("organization:{}", user_org_id);

        {
            let mut org_rooms = self.organization_rooms.write().await;
            org_rooms.entry(room_id.clone())
                .or_insert_with(HashSet::new)
                .insert(client_id.clone());
        }

        println!("User {} joined organization room", user_id);
    }

    // Broadcast to organization
    async fn broadcast_to_organization(&self, organization_id: &str, event: String, data: String) {
        let room_id = format!("organization:{}", organization_id);

        let clients = {
            let org_rooms = self.organization_rooms.read().await;
            org_rooms.get(&room_id).cloned()
        };

        if let Some(client_ids) = clients {
            let senders = self.client_senders.read().await;

            for client_id in client_ids {
                if let Some(sender) = senders.get(&client_id) {
                    let _ = sender.send(Message::Text(data.clone())).await;
                }
            }
        }
    }
}
```

**User-Specific Rooms**:

```rust
impl ConnectionManager {
    // Join user room
    async fn join_user_room(&self, client_id: String, user_id: String) {
        let room_id = format!("user:{}", user_id);

        let mut user_rooms = self.user_rooms.write().await;
        user_rooms.entry(room_id)
            .or_insert_with(HashSet::new)
            .insert(client_id);
    }

    // Send to specific user
    async fn send_to_user(&self, user_id: &str, event: String, data: String) {
        let room_id = format!("user:{}", user_id);

        let clients = {
            let user_rooms = self.user_rooms.read().await;
            user_rooms.get(&room_id).cloned()
        };

        if let Some(client_ids) = clients {
            let senders = self.client_senders.read().await;

            for client_id in client_ids {
                if let Some(sender) = senders.get(&client_id) {
                    let _ = sender.send(Message::Text(data.clone())).await;
                }
            }
        }
    }
}
```

**Project Rooms** (Dynamic):

```rust
#[derive(Debug, Deserialize)]
struct JoinProjectMessage {
    project_id: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ProjectResponse {
    JoinedProject { project_id: String },
    Error { message: String },
}

impl ConnectionManager {
    async fn handle_join_project(
        &self,
        client_id: &str,
        user_id: &str,
        project_id: &str,
        project_service: &ProjectService,
    ) -> Result<(), String> {
        // Verify user has access to project
        let has_access = project_service
            .user_has_access(user_id, project_id)
            .await
            .map_err(|e| e.to_string())?;

        if !has_access {
            return Err("Access denied to project".to_string());
        }

        // Join project room
        let room_id = format!("project:{}", project_id);
        let mut project_rooms = self.organization_rooms.write().await;
        project_rooms.entry(room_id)
            .or_insert_with(HashSet::new)
            .insert(client_id.to_string());

        Ok(())
    }
}
```

---

## Authentication & Authorization

### 1. JWT Authentication for WebSockets

```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use axum::extract::ws::Message;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,        // user_id
    email: String,
    role: String,
    organization_id: String,
    exp: usize,
}

#[derive(Clone)]
struct UserContext {
    user_id: String,
    email: String,
    role: String,
    organization_id: String,
}

struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    fn extract_token_from_query(&self, query: &str) -> Option<String> {
        // Parse query string for token parameter
        for param in query.split('&') {
            let parts: Vec<&str> = param.split('=').collect();
            if parts.len() == 2 && parts[0] == "token" {
                return Some(parts[1].to_string());
            }
        }
        None
    }

    fn extract_token_from_header(&self, auth_header: &str) -> Option<String> {
        if auth_header.starts_with("Bearer ") {
            return Some(auth_header[7..].to_string());
        }
        None
    }
}
```

### 2. Connection Authentication Middleware

```rust
use axum::{
    extract::{Query, ws::WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::Response,
};

async fn authenticated_websocket_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, StatusCode> {
    let auth_service = AuthService::new(state.jwt_secret.clone());

    // Try multiple token sources
    let token = params.get("token")
        .cloned()
        .or_else(|| {
            headers.get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| auth_service.extract_token_from_header(h))
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token
    let claims = auth_service
        .verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_context = UserContext {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        role: claims.role.clone(),
        organization_id: claims.organization_id.clone(),
    };

    Ok(ws.on_upgrade(move |socket| {
        handle_authenticated_socket(socket, user_context, state)
    }))
}

async fn handle_authenticated_socket(
    mut socket: WebSocket,
    user: UserContext,
    state: Arc<AppState>,
) {
    let client_id = uuid::Uuid::new_v4().to_string();

    // Join organization and user rooms
    state.room_manager
        .join_user_room(client_id.clone(), user.user_id.clone())
        .await;

    state.room_manager
        .handle_connection(
            client_id.clone(),
            user.organization_id.clone(),
            user.user_id.clone(),
        )
        .await;

    println!("Authenticated user {} connected", user.email);

    // Handle messages...
    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                // Process authenticated messages
            }
        }
    }
}
```

### 3. Role-Based Authorization

```rust
#[derive(Debug, Clone, PartialEq)]
enum Role {
    OrgOwner,
    Admin,
    User,
}

impl Role {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ORG_OWNER" => Some(Role::OrgOwner),
            "ADMIN" => Some(Role::Admin),
            "USER" => Some(Role::User),
            _ => None,
        }
    }
}

struct AuthorizationService;

impl AuthorizationService {
    fn check_roles(&self, user_role: &str, required_roles: &[Role]) -> Result<(), String> {
        let role = Role::from_str(user_role)
            .ok_or_else(|| "Invalid role".to_string())?;

        if required_roles.contains(&role) {
            Ok(())
        } else {
            Err("Insufficient permissions".to_string())
        }
    }
}

// Usage in message handler
async fn handle_delete_project(
    user: &UserContext,
    project_id: String,
    auth_service: &AuthorizationService,
) -> Result<(), String> {
    // Check authorization
    auth_service.check_roles(&user.role, &[Role::OrgOwner, Role::Admin])?;

    // Only ORG_OWNER and ADMIN can delete projects
    println!("Deleting project {} by user {}", project_id, user.user_id);

    Ok(())
}
```

---

## Message Patterns

### 1. Request-Response Pattern

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RequestMessage {
    GetActiveTimer,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ResponseMessage {
    ActiveTimer { timer: Option<Timer> },
}

#[derive(Debug, Serialize)]
struct Timer {
    id: String,
    project_id: String,
    started_at: String,
}

async fn handle_request_response(
    mut socket: WebSocket,
    user: UserContext,
    timer_service: Arc<TimerService>,
) {
    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<RequestMessage>(&text) {
                        Ok(RequestMessage::GetActiveTimer) => {
                            let timer = timer_service
                                .get_active_timer(&user.user_id)
                                .await
                                .ok();

                            let response = ResponseMessage::ActiveTimer { timer };
                            let response_text = serde_json::to_string(&response).unwrap();
                            let _ = socket.send(Message::Text(response_text)).await;
                        }
                    }
                }
            }
        }
    }
}
```

### 2. Broadcast Pattern

```rust
#[derive(Debug, Deserialize)]
struct BroadcastMessage {
    message: String,
}

struct BroadcastService {
    broadcast_tx: broadcast::Sender<String>,
    clients: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
}

impl BroadcastService {
    // Broadcast to all clients
    async fn broadcast_to_all(&self, message: String) {
        let clients = self.clients.read().await;

        for (client_id, sender) in clients.iter() {
            let msg = Message::Text(message.clone());
            let _ = sender.send(msg).await;
        }
    }

    // Broadcast to all except sender
    async fn broadcast_except_sender(&self, sender_id: &str, message: String) {
        let clients = self.clients.read().await;

        for (client_id, sender) in clients.iter() {
            if client_id != sender_id {
                let msg = Message::Text(message.clone());
                let _ = sender.send(msg).await;
            }
        }
    }
}
```

### 3. Typing Indicator Pattern

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum TypingMessage {
    TypingStart { project_id: String },
    TypingStop { project_id: String },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
enum TypingNotification {
    UserTyping {
        user_id: String,
        user_name: String,
        project_id: String,
    },
    UserStoppedTyping {
        user_id: String,
        project_id: String,
    },
}

struct TypingService {
    room_manager: Arc<RoomManager>,
}

impl TypingService {
    async fn handle_typing_start(
        &self,
        client_id: &str,
        user: &UserContext,
        project_id: String,
    ) {
        let room_id = format!("project:{}", project_id);

        let notification = TypingNotification::UserTyping {
            user_id: user.user_id.clone(),
            user_name: user.email.clone(), // Or fetch full name
            project_id: project_id.clone(),
        };

        // Broadcast to all in room except sender
        self.room_manager
            .broadcast_to_room_except(&room_id, client_id, notification)
            .await;
    }

    async fn handle_typing_stop(
        &self,
        client_id: &str,
        user: &UserContext,
        project_id: String,
    ) {
        let room_id = format!("project:{}", project_id);

        let notification = TypingNotification::UserStoppedTyping {
            user_id: user.user_id.clone(),
            project_id: project_id.clone(),
        };

        // Broadcast to all in room except sender
        self.room_manager
            .broadcast_to_room_except(&room_id, client_id, notification)
            .await;
    }
}

// Client-side handling remains the same (React)
```

### 4. Presence Pattern

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
enum PresenceNotification {
    UserOnline { user_id: String, email: String },
    UserOffline { user_id: String },
    UsersOnline { user_ids: Vec<String> },
}

struct PresenceService {
    online_users: Arc<RwLock<HashMap<String, HashSet<String>>>>, // org_id → user_ids
    room_manager: Arc<RoomManager>,
}

impl PresenceService {
    fn new(room_manager: Arc<RoomManager>) -> Self {
        Self {
            online_users: Arc::new(RwLock::new(HashMap::new())),
            room_manager,
        }
    }

    async fn handle_connection(&self, user: &UserContext, client_id: &str) {
        // Add user to online set
        {
            let mut online = self.online_users.write().await;
            online
                .entry(user.organization_id.clone())
                .or_insert_with(HashSet::new)
                .insert(user.user_id.clone());
        }

        // Notify organization members (except new user)
        let notification = PresenceNotification::UserOnline {
            user_id: user.user_id.clone(),
            email: user.email.clone(),
        };

        let room_id = format!("organization:{}", user.organization_id);
        self.room_manager
            .broadcast_to_room_except(&room_id, client_id, notification)
            .await;

        // Send current online users to new connection
        let online_user_ids = {
            let online = self.online_users.read().await;
            online
                .get(&user.organization_id)
                .map(|users| users.iter().cloned().collect())
                .unwrap_or_else(Vec::new)
        };

        let users_online = PresenceNotification::UsersOnline {
            user_ids: online_user_ids,
        };

        self.room_manager
            .send_to_client(client_id, users_online)
            .await;
    }

    async fn handle_disconnect(&self, user: &UserContext, client_id: &str) {
        // Remove from online set
        {
            let mut online = self.online_users.write().await;
            if let Some(users) = online.get_mut(&user.organization_id) {
                users.remove(&user.user_id);
            }
        }

        // Notify organization members
        let notification = PresenceNotification::UserOffline {
            user_id: user.user_id.clone(),
        };

        let room_id = format!("organization:{}", user.organization_id);
        self.room_manager
            .broadcast_to_room_except(&room_id, client_id, notification)
            .await;
    }
}
```

---

## Scaling WebSockets

### 1. Redis Pub/Sub for Multi-Server

```rust
use redis::{aio::Connection, AsyncCommands, Client};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedisMessage {
    event: String,
    data: serde_json::Value,
    room: Option<String>,
}

struct RedisWebSocketAdapter {
    redis_client: Client,
    local_broadcast_tx: broadcast::Sender<RedisMessage>,
}

impl RedisWebSocketAdapter {
    async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let (tx, _rx) = broadcast::channel(1000);

        Ok(Self {
            redis_client: client,
            local_broadcast_tx: tx,
        })
    }

    // Subscribe to Redis channel for incoming messages from other servers
    async fn subscribe_to_redis(&self) -> Result<(), redis::RedisError> {
        let mut pubsub = self.redis_client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe("ws:broadcast").await?;

        let tx = self.local_broadcast_tx.clone();

        tokio::spawn(async move {
            loop {
                match pubsub.on_message().next().await {
                    Some(msg) => {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            if let Ok(redis_msg) = serde_json::from_str::<RedisMessage>(&payload) {
                                // Broadcast to local WebSocket clients
                                let _ = tx.send(redis_msg);
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        Ok(())
    }

    // Publish message to Redis for distribution to other servers
    async fn publish_to_redis(
        &self,
        event: String,
        data: serde_json::Value,
        room: Option<String>,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;

        let message = RedisMessage { event, data, room };
        let payload = serde_json::to_string(&message).unwrap();

        conn.publish("ws:broadcast", payload).await?;

        Ok(())
    }
}

// Usage in main
async fn setup_websocket_cluster() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let adapter = RedisWebSocketAdapter::new(&redis_url).await?;
    adapter.subscribe_to_redis().await?;

    // Use adapter in WebSocket handlers
    Ok(())
}
```

### 2. Sticky Sessions (Load Balancer)

**Nginx Configuration**:

```nginx
upstream websocket_backend {
    ip_hash;  # Sticky sessions based on IP
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    server 127.0.0.1:3003;
}

server {
    listen 80;

    location /ws {
        proxy_pass http://websocket_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;  # 24 hours for long-lived connections
    }

    location /time-tracking {
        proxy_pass http://websocket_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;
    }

    location /notifications {
        proxy_pass http://websocket_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;
    }
}
```

### 3. Horizontal Scaling Strategy

```rust
use redis::Client;
use tokio::sync::RwLock;

struct WebSocketScaler {
    redis_client: Client,
    connection_manager: Arc<ConnectionManager>,
}

impl WebSocketScaler {
    async fn new(
        redis_url: &str,
        connection_manager: Arc<ConnectionManager>,
    ) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;

        let scaler = Self {
            redis_client: client,
            connection_manager,
        };

        scaler.subscribe_to_redis().await?;

        Ok(scaler)
    }

    async fn subscribe_to_redis(&self) -> Result<(), redis::RedisError> {
        let mut pubsub = self.redis_client.get_async_connection().await?.into_pubsub();
        pubsub.subscribe("ws:broadcast").await?;

        let conn_manager = self.connection_manager.clone();

        tokio::spawn(async move {
            loop {
                match pubsub.on_message().next().await {
                    Some(msg) => {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            if let Ok(redis_msg) = serde_json::from_str::<RedisMessage>(&payload) {
                                // Route message to appropriate room or broadcast to all
                                if let Some(room) = redis_msg.room {
                                    conn_manager
                                        .broadcast_to_room(&room, redis_msg.event, redis_msg.data.to_string())
                                        .await;
                                } else {
                                    conn_manager
                                        .broadcast_to_all(redis_msg.event, redis_msg.data.to_string())
                                        .await;
                                }
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        Ok(())
    }

    async fn broadcast_across_servers(
        &self,
        event: String,
        data: serde_json::Value,
        room: Option<String>,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;

        let message = RedisMessage { event, data, room };
        let payload = serde_json::to_string(&message).unwrap();

        conn.publish("ws:broadcast", payload).await?;

        Ok(())
    }
}
```

---

## Error Handling & Reconnection

### 1. Client-Side Reconnection

```typescript
export function useSocket({ url, token }: UseSocketOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    const maxReconnectAttempts = Infinity;
    let currentAttempts = 0;

    const connect = () => {
      const wsUrl = token ? `${url}?token=${token}` : url;
      const socket = new WebSocket(wsUrl);

      socket.onopen = () => {
        console.log('Connected');
        setIsConnected(true);
        setReconnectAttempts(0);
        currentAttempts = 0;
      };

      socket.onclose = (event) => {
        console.log('Disconnected:', event.reason);
        setIsConnected(false);

        // Automatically reconnect
        if (currentAttempts < maxReconnectAttempts) {
          currentAttempts++;
          setReconnectAttempts(currentAttempts);

          const delay = Math.min(1000 * Math.pow(2, currentAttempts - 1), 5000);
          console.log(`Reconnection attempt ${currentAttempts} in ${delay}ms`);

          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, delay);
        } else {
          console.error('Failed to reconnect');
        }
      };

      socket.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      socketRef.current = socket;
    };

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (socketRef.current) {
        socketRef.current.close();
      }
    };
  }, [url, token]);

  return { socket: socketRef.current, isConnected, reconnectAttempts };
}
```

### 2. Server-Side Error Handling

```rust
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OperationResponse {
    Success { result: serde_json::Value },
    Error { message: String, code: String },
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
    timestamp: String,
}

async fn handle_risky_operation(
    mut socket: WebSocket,
    data: serde_json::Value,
    risky_service: Arc<RiskyService>,
) {
    match risky_service.perform_operation(data).await {
        Ok(result) => {
            let response = OperationResponse::Success { result };
            let msg = serde_json::to_string(&response).unwrap();
            let _ = socket.send(Message::Text(msg)).await;
        }
        Err(e) => {
            eprintln!("Operation failed: {:?}", e);

            let response = OperationResponse::Error {
                message: e.to_string(),
                code: "OPERATION_FAILED".to_string(),
            };
            let msg = serde_json::to_string(&response).unwrap();
            let _ = socket.send(Message::Text(msg)).await;
        }
    }
}

// Global error handler wrapper
async fn handle_socket_with_error_handling(
    mut socket: WebSocket,
    state: Arc<AppState>,
) {
    loop {
        tokio::select! {
            Some(result) = socket.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = process_message(msg, &mut socket, &state).await {
                            eprintln!("WebSocket exception: {:?}", e);

                            let error_response = ErrorResponse {
                                message: "An error occurred".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };

                            let error_msg = serde_json::to_string(&error_response).unwrap();
                            let _ = socket.send(Message::Text(error_msg)).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Socket receive error: {:?}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn process_message(
    msg: Message,
    socket: &mut WebSocket,
    state: &Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Process message with proper error propagation
    match msg {
        Message::Text(text) => {
            // Handle text message
            Ok(())
        }
        Message::Close(_) => {
            Err("Connection closed".into())
        }
        _ => Ok(()),
    }
}
```

---

## Performance Optimization

### 1. Message Throttling/Debouncing

```typescript
// Client-side throttling
import { throttle } from 'lodash';

const emitMouseMove = throttle((socket, position) => {
  socket.emit('mouse:move', position);
}, 100); // Max 10 messages per second

function handleMouseMove(event) {
  emitMouseMove(socket, { x: event.clientX, y: event.clientY });
}
```

### 2. Binary Data

```rust
use tokio::fs;

async fn handle_image_request(mut socket: WebSocket) {
    // Read image file
    match fs::read("path/to/image.png").await {
        Ok(image_buffer) => {
            // Send as binary message
            let _ = socket.send(Message::Binary(image_buffer)).await;
        }
        Err(e) => {
            eprintln!("Failed to read image: {:?}", e);
        }
    }
}

// Client handling remains the same (JavaScript)
```

### 3. Compression

```rust
use axum::extract::ws::WebSocketUpgrade;
use tungstenite::protocol::WebSocketConfig;

// Configure WebSocket with compression
fn create_websocket_config() -> WebSocketConfig {
    WebSocketConfig {
        max_send_queue: Some(512),
        max_message_size: Some(64 << 20), // 64 MB
        max_frame_size: Some(16 << 20),   // 16 MB
        accept_unmasked_frames: false,
        // Note: Per-message compression is handled by the tungstenite library
        // Enable it in Cargo.toml with feature flags if needed
        ..Default::default()
    }
}

// For message-level compression, use explicit compression:
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

async fn send_compressed_message(socket: &mut WebSocket, data: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Only compress large messages (> 1KB)
    if data.len() > 1024 {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data.as_bytes())?;
        let compressed = encoder.finish()?;

        socket.send(Message::Binary(compressed)).await?;
    } else {
        socket.send(Message::Text(data.to_string())).await?;
    }

    Ok(())
}
```

---

## Testing WebSockets

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_room_subscription() {
        let room_manager = Arc::new(RoomManager::new());
        let client_id = "test-client-id";
        let user_org_id = "org-123";
        let room = "organization:org-123";

        // Subscribe to room
        let result = room_manager.subscribe(client_id, room, user_org_id).await;

        assert!(result.is_ok());

        // Verify client is in room
        let rooms = room_manager.rooms.read().await;
        assert!(rooms.get(room).unwrap().contains(client_id));
    }

    #[tokio::test]
    async fn test_unauthorized_room_access() {
        let room_manager = Arc::new(RoomManager::new());
        let client_id = "test-client-id";
        let user_org_id = "org-123";
        let room = "organization:org-456"; // Different org

        // Attempt to subscribe to different org
        let result = room_manager.subscribe(client_id, room, user_org_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Access denied to this room");
    }

    #[tokio::test]
    async fn test_cleanup_client() {
        let room_manager = Arc::new(RoomManager::new());
        let client_id = "test-client-id";
        let user_org_id = "org-123";

        // Subscribe to multiple rooms
        room_manager.subscribe(client_id, "room1", user_org_id).await.unwrap();
        room_manager.subscribe(client_id, "room2", user_org_id).await.unwrap();

        // Cleanup
        room_manager.cleanup_client(client_id).await;

        // Verify client removed from all rooms
        let rooms = room_manager.rooms.read().await;
        assert!(!rooms.get("room1").map_or(false, |r| r.contains(client_id)));
        assert!(!rooms.get("room2").map_or(false, |r| r.contains(client_id)));
    }
}
```

### 2. Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio_tungstenite::connect_async;
    use futures_util::{StreamExt, SinkExt};

    #[tokio::test]
    async fn test_websocket_connection() {
        // Start server in background
        let server = tokio::spawn(async {
            let app = create_router();
            axum::Server::bind(&"127.0.0.1:3001".parse().unwrap())
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get auth token
        let token = get_test_auth_token();

        // Connect client
        let url = format!("ws://localhost:3001/ws?token={}", token);
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (mut write, mut read) = ws_stream.split();

        // Send message
        let msg = serde_json::json!({
            "type": "subscribe",
            "room": "organization:test-org"
        });
        write.send(tungstenite::Message::Text(msg.to_string()))
            .await
            .unwrap();

        // Receive response
        if let Some(Ok(tungstenite::Message::Text(text))) = read.next().await {
            let response: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(response["type"], "subscribed");
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_room() {
        // Setup multiple clients in same room
        // Send message from one client
        // Verify other clients receive it
        // Implementation details...
    }

    fn get_test_auth_token() -> String {
        // Generate test JWT token
        "test-token".to_string()
    }
}
```

---

## Summary

### WebSocket Best Practices Checklist

#### ✅ Connection Management

- [ ] Implement authentication on connection
- [ ] Auto-reconnect on disconnect
- [ ] Handle connection errors gracefully
- [ ] Track connection state
- [ ] Clean up on unmount/disconnect

#### ✅ Message Handling

- [ ] Validate all incoming messages
- [ ] Throttle/debounce high-frequency events
- [ ] Use binary for large payloads
- [ ] Implement request-response pattern
- [ ] Handle message errors

#### ✅ Scaling

- [ ] Use Redis adapter for multi-server
- [ ] Implement sticky sessions
- [ ] Compress messages > 1KB
- [ ] Monitor active connections
- [ ] Rate limit messages per client

#### ✅ Security

- [ ] Authenticate on connection
- [ ] Authorize room access
- [ ] Sanitize all inputs
- [ ] Implement rate limiting
- [ ] Use secure WebSocket (wss://)

#### ✅ Testing

- [ ] Unit test event handlers
- [ ] Integration test workflows
- [ ] Load test concurrent connections
- [ ] Test reconnection logic
- [ ] Test error scenarios

---

## Related Patterns

- **Pattern 12**: [Observer Pattern](./12-Observer-Pattern.md)
- **Pattern 39**: [Security Patterns Guide](./39-Security-Patterns-Guide.md)
- **Pattern 42**: [GraphQL API Patterns](./42-GraphQL-API-Patterns.md)

---

## References

- [Axum WebSocket Documentation](https://docs.rs/axum/latest/axum/extract/ws/index.html)
- [Tokio-Tungstenite](https://docs.rs/tokio-tungstenite/)
- [WebSocket Protocol (RFC 6455)](https://tools.ietf.org/html/rfc6455)
- [Redis Pub/Sub](https://redis.io/docs/manual/pubsub/)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Last Updated**: October 8, 2025
**Version**: 1.0
**Status**: Active
