# Phase 6: Real-time Communication & Audit Trail

**Duration**: Weeks 11-12 (14 days)
**Goal**: WebSocket Server → Real-time Team Status → Audit Trail System → Message History
**Dependencies**: Phase 5 complete (Frontend basics)

---

## Epic 1: WebSocket Server Implementation

### Task 1.1: Tokio-Tungstenite WebSocket Server

**Type**: Backend
**Dependencies**: Axum server running

**Subtasks**:

- [ ] 1.1.1: Add WebSocket dependencies

```toml
# apps/api/Cargo.toml
[dependencies]
tokio-tungstenite = "0.21"
futures-util = "0.3"
dashmap = "5.5"
```

- [ ] 1.1.2: Create WebSocket connection manager

```rust
// apps/api/src/realtime/connection_manager.rs
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub type Sender = mpsc::UnboundedSender<Message>;
pub type Receiver = mpsc::UnboundedReceiver<Message>;

#[derive(Clone)]
pub struct ConnectionManager {
    // team_id -> user_id -> sender
    connections: Arc<DashMap<Uuid, DashMap<Uuid, Sender>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    pub fn add_connection(&self, team_id: Uuid, user_id: Uuid, sender: Sender) {
        self.connections
            .entry(team_id)
            .or_insert_with(DashMap::new)
            .insert(user_id, sender);

        tracing::info!(
            "WebSocket connected: team={}, user={}, total_connections={}",
            team_id,
            user_id,
            self.count_connections()
        );
    }

    pub fn remove_connection(&self, team_id: Uuid, user_id: Uuid) {
        if let Some(team_conns) = self.connections.get(&team_id) {
            team_conns.remove(&user_id);
            if team_conns.is_empty() {
                drop(team_conns);
                self.connections.remove(&team_id);
            }
        }

        tracing::info!(
            "WebSocket disconnected: team={}, user={}, remaining={}",
            team_id,
            user_id,
            self.count_connections()
        );
    }

    pub async fn broadcast_to_team(&self, team_id: Uuid, message: Message) {
        if let Some(team_conns) = self.connections.get(&team_id) {
            for entry in team_conns.iter() {
                let user_id = entry.key();
                let sender = entry.value();

                if let Err(e) = sender.send(message.clone()) {
                    tracing::error!(
                        "Failed to send message to user {}: {}",
                        user_id,
                        e
                    );
                }
            }
        }
    }

    pub async fn send_to_user(&self, team_id: Uuid, user_id: Uuid, message: Message) -> bool {
        if let Some(team_conns) = self.connections.get(&team_id) {
            if let Some(sender) = team_conns.get(&user_id) {
                return sender.send(message).is_ok();
            }
        }
        false
    }

    pub fn count_connections(&self) -> usize {
        self.connections
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }

    pub fn get_team_users(&self, team_id: Uuid) -> Vec<Uuid> {
        self.connections
            .get(&team_id)
            .map(|team| team.iter().map(|e| *e.key()).collect())
            .unwrap_or_default()
    }
}
```

- [ ] 1.1.3: Create WebSocket handler

```rust
// apps/api/src/realtime/handler.rs
use crate::api::auth::jwt::Claims;
use crate::realtime::connection_manager::ConnectionManager;
use crate::realtime::protocol::{ClientMessage, ServerMessage};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<Claims>,
    State(manager): State<ConnectionManager>,
) -> Response {
    let user_id = Uuid::parse_str(&claims.sub).unwrap();
    let company_id = Uuid::parse_str(&claims.company_id).unwrap();

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, company_id, manager))
}

async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    company_id: Uuid,
    manager: ConnectionManager,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Initially not connected to any team
    let mut current_team_id: Option<Uuid> = None;

    // Spawn task to forward messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_message(
                            client_msg,
                            user_id,
                            company_id,
                            &mut current_team_id,
                            &manager,
                            &tx,
                        )
                        .await;
                    }
                }
                Message::Close(_) => break,
                Message::Ping(data) => {
                    let _ = tx.send(Message::Pong(data));
                }
                _ => {}
            }
        }

        // Clean up on disconnect
        if let Some(team_id) = current_team_id {
            manager.remove_connection(team_id, user_id);
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn handle_client_message(
    msg: ClientMessage,
    user_id: Uuid,
    company_id: Uuid,
    current_team_id: &mut Option<Uuid>,
    manager: &ConnectionManager,
    sender: &mpsc::UnboundedSender<Message>,
) {
    match msg {
        ClientMessage::Subscribe { team_id } => {
            // TODO: Verify user has access to this team via DB query

            // Remove from previous team if any
            if let Some(old_team) = current_team_id.take() {
                manager.remove_connection(old_team, user_id);
            }

            // Add to new team
            manager.add_connection(team_id, user_id, sender.clone());
            *current_team_id = Some(team_id);

            let response = ServerMessage::Subscribed { team_id };
            let _ = sender.send(Message::Text(serde_json::to_string(&response).unwrap()));
        }

        ClientMessage::Unsubscribe => {
            if let Some(team_id) = current_team_id.take() {
                manager.remove_connection(team_id, user_id);

                let response = ServerMessage::Unsubscribed;
                let _ = sender.send(Message::Text(serde_json::to_string(&response).unwrap()));
            }
        }

        ClientMessage::Ping => {
            let response = ServerMessage::Pong;
            let _ = sender.send(Message::Text(serde_json::to_string(&response).unwrap()));
        }
    }
}
```

- [ ] 1.1.4: Define WebSocket protocol

```rust
// apps/api/src/realtime/protocol.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe { team_id: Uuid },
    Unsubscribe,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // Connection management
    Subscribed { team_id: Uuid },
    Unsubscribed,
    Pong,

    // Team status updates
    TeamStatusChanged {
        team_id: Uuid,
        new_status: String,
        timestamp: DateTime<Utc>,
    },

    // Task updates
    TaskCreated {
        task_id: Uuid,
        team_id: Uuid,
        title: String,
        assigned_to: Option<Uuid>,
    },

    TaskStatusChanged {
        task_id: Uuid,
        new_status: String,
        timestamp: DateTime<Utc>,
    },

    TaskAssigned {
        task_id: Uuid,
        worker_id: Uuid,
        worker_name: String,
    },

    TaskCompleted {
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    },

    // Agent updates
    AgentStatusChanged {
        agent_id: Uuid,
        new_status: String,
        current_workload: i32,
    },

    // Messages
    NewMessage {
        message_id: Uuid,
        from_agent_id: Uuid,
        from_agent_name: String,
        message_type: String,
        content: String,
        timestamp: DateTime<Utc>,
    },

    // Cost tracking
    CostUpdate {
        team_id: Uuid,
        total_cost: f64,
        last_operation_cost: f64,
    },

    // Errors
    Error {
        code: String,
        message: String,
    },
}
```

- [ ] 1.1.5: Add WebSocket route to Axum

```rust
// apps/api/src/main.rs (add to router)
use crate::realtime::handler::websocket_handler;
use crate::realtime::connection_manager::ConnectionManager;

let manager = ConnectionManager::new();

let app = Router::new()
    // ... existing routes
    .route("/ws", get(websocket_handler))
    .layer(middleware::from_fn_with_state(
        jwt_service.clone(),
        auth_middleware
    ))
    .with_state(manager.clone())
    .with_state(app_state);
```

- [ ] 1.1.6: Test WebSocket connection

```bash
# Install websocat for testing
brew install websocat  # macOS
# or
cargo install websocat

# Get a JWT token first
TOKEN=$(curl -X POST http://localhost:4000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' \
  | jq -r '.token')

# Connect to WebSocket
websocat "ws://localhost:4000/ws" \
  --header "Authorization: Bearer $TOKEN"

# Send subscription message
{"type":"subscribe","team_id":"00000000-0000-0000-0000-000000000001"}
```

**Acceptance Criteria**:

- [ ] WebSocket server accepts connections
- [ ] JWT authentication works for WebSocket
- [ ] Clients can subscribe to team updates
- [ ] Clients can unsubscribe
- [ ] Connection manager tracks all connections
- [ ] Ping/pong keeps connections alive
- [ ] Clean disconnection handling

---

### Task 1.2: WebSocket Broadcasting Service

**Type**: Backend
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 1.2.1: Create broadcast service

```rust
// apps/api/src/realtime/broadcast.rs
use crate::realtime::connection_manager::ConnectionManager;
use crate::realtime::protocol::ServerMessage;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Clone)]
pub struct BroadcastService {
    manager: ConnectionManager,
}

impl BroadcastService {
    pub fn new(manager: ConnectionManager) -> Self {
        Self { manager }
    }

    pub async fn broadcast_team_status_change(
        &self,
        team_id: Uuid,
        new_status: String,
    ) {
        let msg = ServerMessage::TeamStatusChanged {
            team_id,
            new_status,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }

    pub async fn broadcast_task_created(
        &self,
        team_id: Uuid,
        task_id: Uuid,
        title: String,
        assigned_to: Option<Uuid>,
    ) {
        let msg = ServerMessage::TaskCreated {
            task_id,
            team_id,
            title,
            assigned_to,
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }

    pub async fn broadcast_task_status_change(
        &self,
        team_id: Uuid,
        task_id: Uuid,
        new_status: String,
    ) {
        let msg = ServerMessage::TaskStatusChanged {
            task_id,
            new_status,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }

    pub async fn broadcast_agent_status_change(
        &self,
        team_id: Uuid,
        agent_id: Uuid,
        new_status: String,
        current_workload: i32,
    ) {
        let msg = ServerMessage::AgentStatusChanged {
            agent_id,
            new_status,
            current_workload,
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }

    pub async fn broadcast_new_message(
        &self,
        team_id: Uuid,
        message_id: Uuid,
        from_agent_id: Uuid,
        from_agent_name: String,
        message_type: String,
        content: String,
    ) {
        let msg = ServerMessage::NewMessage {
            message_id,
            from_agent_id,
            from_agent_name,
            message_type,
            content,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }

    pub async fn broadcast_cost_update(
        &self,
        team_id: Uuid,
        total_cost: f64,
        last_operation_cost: f64,
    ) {
        let msg = ServerMessage::CostUpdate {
            team_id,
            total_cost,
            last_operation_cost,
        };

        let json = serde_json::to_string(&msg).unwrap();
        self.manager.broadcast_to_team(team_id, Message::Text(json)).await;
    }
}
```

- [ ] 1.2.2: Integrate with existing services

```rust
// apps/api/src/services/team_service.rs
use crate::realtime::broadcast::BroadcastService;

pub struct TeamService {
    repository: TeamsRepository,
    broadcast: BroadcastService,
}

impl TeamService {
    pub async fn update_team_status(
        &self,
        team_id: Uuid,
        new_status: TeamStatus,
    ) -> Result<(), ServiceError> {
        self.repository.update_status(team_id, &new_status.to_string()).await?;

        // Broadcast to WebSocket clients
        self.broadcast
            .broadcast_team_status_change(team_id, new_status.to_string())
            .await;

        Ok(())
    }
}
```

- [ ] 1.2.3: Integrate with task service

```rust
// apps/api/src/services/task_service.rs
impl TaskService {
    pub async fn assign_task(
        &self,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> Result<(), ServiceError> {
        let mut task = self.repository.find_by_id(task_id).await?
            .ok_or(ServiceError::NotFound)?;

        task.assign(worker_id, self.manager_id);
        self.repository.update(&task).await?;

        // Broadcast assignment
        let worker = self.get_worker_name(worker_id).await?;
        self.broadcast
            .broadcast_task_assigned(task.team_id, task_id, worker_id, worker)
            .await;

        Ok(())
    }
}
```

**Acceptance Criteria**:

- [ ] Broadcast service integrated with team operations
- [ ] Status changes broadcast to all subscribers
- [ ] Task updates broadcast in real-time
- [ ] Agent status changes broadcast
- [ ] No message loss under normal conditions

---

## Epic 2: WebSocket Client (Next.js)

### Task 2.1: React WebSocket Hook

**Type**: Frontend
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 2.1.1: Create WebSocket context

```typescript
// apps/frontend/src/contexts/WebSocketContext.tsx
'use client';

import { createContext, useContext, useEffect, useRef, useState } from 'react';
import { useAuth } from './AuthContext';

interface ServerMessage {
  type: string;
  [key: string]: any;
}

interface WebSocketContextType {
  isConnected: boolean;
  subscribe: (teamId: string) => void;
  unsubscribe: () => void;
  sendMessage: (message: any) => void;
  lastMessage: ServerMessage | null;
}

const WebSocketContext = createContext<WebSocketContextType | null>(null);

export function WebSocketProvider({ children }: { children: React.ReactNode }) {
  const { token } = useAuth();
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<ServerMessage | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    if (!token) return;

    const connect = () => {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
      };

      ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        setLastMessage(message);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);

        // Attempt reconnect after 3 seconds
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('Attempting to reconnect...');
          connect();
        }, 3000);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        ws.close();
      };

      wsRef.current = ws;
    };

    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      wsRef.current?.close();
    };
  }, [token]);

  const subscribe = (teamId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe',
        team_id: teamId,
      }));
    }
  };

  const unsubscribe = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe',
      }));
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    }
  };

  return (
    <WebSocketContext.Provider
      value={{ isConnected, subscribe, unsubscribe, sendMessage, lastMessage }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export const useWebSocket = () => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within WebSocketProvider');
  }
  return context;
};
```

- [ ] 2.1.2: Create typed message handlers

```typescript
// apps/frontend/src/hooks/useTeamUpdates.ts
import { useEffect } from 'react';
import { useWebSocket } from '@/contexts/WebSocketContext';
import { useQueryClient } from '@tanstack/react-query';

export function useTeamUpdates(teamId: string | undefined) {
  const { subscribe, unsubscribe, lastMessage } = useWebSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!teamId) return;

    subscribe(teamId);

    return () => {
      unsubscribe();
    };
  }, [teamId, subscribe, unsubscribe]);

  useEffect(() => {
    if (!lastMessage) return;

    switch (lastMessage.type) {
      case 'team_status_changed':
        queryClient.invalidateQueries({ queryKey: ['team', teamId] });
        break;

      case 'task_created':
        queryClient.invalidateQueries({ queryKey: ['tasks', teamId] });
        break;

      case 'task_status_changed':
        queryClient.invalidateQueries({ queryKey: ['tasks', teamId] });
        queryClient.invalidateQueries({ queryKey: ['task', lastMessage.task_id] });
        break;

      case 'agent_status_changed':
        queryClient.invalidateQueries({ queryKey: ['team-members', teamId] });
        break;

      case 'new_message':
        queryClient.invalidateQueries({ queryKey: ['messages', teamId] });
        break;

      case 'cost_update':
        queryClient.invalidateQueries({ queryKey: ['costs', teamId] });
        break;
    }
  }, [lastMessage, teamId, queryClient]);
}
```

- [ ] 2.1.3: Add connection status indicator

```typescript
// apps/frontend/src/components/ConnectionStatus.tsx
'use client';

import { useWebSocket } from '@/contexts/WebSocketContext';
import { Wifi, WifiOff } from 'lucide-react';

export function ConnectionStatus() {
  const { isConnected } = useWebSocket();

  return (
    <div className="flex items-center gap-2">
      {isConnected ? (
        <>
          <Wifi className="h-4 w-4 text-green-500" />
          <span className="text-sm text-green-600">Connected</span>
        </>
      ) : (
        <>
          <WifiOff className="h-4 w-4 text-red-500" />
          <span className="text-sm text-red-600">Disconnected</span>
        </>
      )}
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] WebSocket connects on app load
- [ ] Auto-reconnects on disconnect
- [ ] Subscribe/unsubscribe works
- [ ] Messages trigger React Query invalidation
- [ ] Connection status displays correctly

---

## Epic 3: Real-time Team Status Broadcast

### Task 3.1: Live Team Dashboard Updates

**Type**: Fullstack
**Dependencies**: Tasks 1.2 and 2.1 complete

**Subtasks**:

- [ ] 3.1.1: Add live status badges

```typescript
// apps/frontend/src/components/team/TeamStatusBadge.tsx
'use client';

import { Badge } from '@/components/ui/badge';
import { useTeamUpdates } from '@/hooks/useTeamUpdates';
import { useQuery } from '@tanstack/react-query';
import { motion } from 'framer-motion';

export function TeamStatusBadge({ teamId }: { teamId: string }) {
  useTeamUpdates(teamId);

  const { data: team } = useQuery({
    queryKey: ['team', teamId],
    queryFn: () => fetchTeam(teamId),
  });

  const statusColors = {
    pending: 'bg-gray-500',
    planning: 'bg-blue-500',
    active: 'bg-green-500 animate-pulse',
    completed: 'bg-green-700',
    failed: 'bg-red-500',
    archived: 'bg-gray-400',
  };

  return (
    <motion.div
      initial={{ scale: 0.9 }}
      animate={{ scale: 1 }}
      transition={{ duration: 0.2 }}
    >
      <Badge className={statusColors[team?.status || 'pending']}>
        {team?.status || 'Loading...'}
      </Badge>
    </motion.div>
  );
}
```

- [ ] 3.1.2: Add live agent status grid

```typescript
// apps/frontend/src/components/team/AgentStatusGrid.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { useTeamUpdates } from '@/hooks/useTeamUpdates';
import { Card } from '@/components/ui/card';
import { Avatar } from '@/components/ui/avatar';
import { motion, AnimatePresence } from 'framer-motion';

export function AgentStatusGrid({ teamId }: { teamId: string }) {
  useTeamUpdates(teamId);

  const { data: members } = useQuery({
    queryKey: ['team-members', teamId],
    queryFn: () => fetchTeamMembers(teamId),
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'border-green-500';
      case 'idle': return 'border-yellow-500';
      case 'busy': return 'border-red-500';
      case 'offline': return 'border-gray-500';
      default: return 'border-gray-300';
    }
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <AnimatePresence>
        {members?.map((member) => (
          <motion.div
            key={member.id}
            layout
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
          >
            <Card className={`p-4 border-2 ${getStatusColor(member.status)}`}>
              <div className="flex items-center gap-3">
                <Avatar className="relative">
                  <div className="w-full h-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center text-white font-bold">
                    {member.specialization[0]}
                  </div>
                  <div className={`absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-white ${
                    member.status === 'busy' ? 'bg-red-500' : 'bg-green-500'
                  }`} />
                </Avatar>

                <div className="flex-1">
                  <h3 className="font-semibold">{member.specialization}</h3>
                  <p className="text-sm text-gray-500">
                    {member.current_workload}/{member.max_concurrent_tasks} tasks
                  </p>
                </div>
              </div>
            </Card>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
```

- [ ] 3.1.3: Add live task progress bar

```typescript
// apps/frontend/src/components/team/TaskProgressBar.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { useTeamUpdates } from '@/hooks/useTeamUpdates';
import { Progress } from '@/components/ui/progress';
import { motion } from 'framer-motion';

export function TaskProgressBar({ teamId }: { teamId: string }) {
  useTeamUpdates(teamId);

  const { data: tasks } = useQuery({
    queryKey: ['tasks', teamId],
    queryFn: () => fetchTasks(teamId),
  });

  const completed = tasks?.filter(t => t.status === 'completed').length || 0;
  const total = tasks?.length || 0;
  const percentage = total > 0 ? (completed / total) * 100 : 0;

  return (
    <div className="space-y-2">
      <div className="flex justify-between text-sm">
        <span className="font-medium">Progress</span>
        <span className="text-gray-500">
          {completed}/{total} tasks completed
        </span>
      </div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
      >
        <Progress value={percentage} className="h-2" />
      </motion.div>
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] Team status updates in real-time
- [ ] Agent status changes reflect immediately
- [ ] Task progress updates live
- [ ] Smooth animations on updates
- [ ] No flashing or jarring transitions

---

## Epic 4: Audit Trail Data Model

### Task 4.1: Create Audit Trail Schema

**Type**: Database
**Dependencies**: Existing database schema

**Subtasks**:

- [ ] 4.1.1: Create audit_events table migration

```sql
-- migrations/XXXXXX_create_audit_events.sql
CREATE TYPE event_type AS ENUM (
    'team_created',
    'team_started',
    'team_completed',
    'team_failed',
    'task_created',
    'task_assigned',
    'task_started',
    'task_completed',
    'task_failed',
    'task_revision_requested',
    'agent_joined',
    'agent_status_changed',
    'message_sent',
    'llm_call_started',
    'llm_call_completed',
    'llm_call_failed',
    'tool_executed',
    'checkpoint_created',
    'cost_incurred',
    'error_occurred'
);

CREATE TYPE event_severity AS ENUM ('info', 'warning', 'error', 'critical');

CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    agent_id UUID,
    event_type event_type NOT NULL,
    severity event_severity NOT NULL DEFAULT 'info',
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_events_team_id ON audit_events(team_id);
CREATE INDEX idx_audit_events_task_id ON audit_events(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_audit_events_agent_id ON audit_events(agent_id) WHERE agent_id IS NOT NULL;
CREATE INDEX idx_audit_events_type ON audit_events(event_type);
CREATE INDEX idx_audit_events_severity ON audit_events(severity);
CREATE INDEX idx_audit_events_created_at ON audit_events(created_at DESC);
CREATE INDEX idx_audit_events_metadata ON audit_events USING gin(metadata);
```

- [ ] 4.1.2: Run migration

```bash
sqlx migrate run --database-url "${DATABASE_URL}"
```

- [ ] 4.1.3: Create Rust audit event model

```rust
// apps/api/src/domain/audit/event.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TeamCreated,
    TeamStarted,
    TeamCompleted,
    TeamFailed,
    TaskCreated,
    TaskAssigned,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskRevisionRequested,
    AgentJoined,
    AgentStatusChanged,
    MessageSent,
    LlmCallStarted,
    LlmCallCompleted,
    LlmCallFailed,
    ToolExecuted,
    CheckpointCreated,
    CostIncurred,
    ErrorOccurred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub team_id: Uuid,
    pub task_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    #[sqlx(try_from = "String")]
    pub event_type: EventType,
    #[sqlx(try_from = "String")]
    pub severity: EventSeverity,
    pub description: String,
    #[sqlx(json)]
    pub metadata: serde_json::Value,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        team_id: Uuid,
        event_type: EventType,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            task_id: None,
            agent_id: None,
            event_type,
            severity: EventSeverity::Info,
            description,
            metadata: serde_json::json!({}),
            user_id: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_task(mut self, task_id: Uuid) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_agent(mut self, agent_id: Uuid) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}
```

- [ ] 4.1.4: Create audit repository

```rust
// apps/api/src/infrastructure/database/repositories/audit.rs
use crate::domain::audit::event::{AuditEvent, EventType, EventSeverity};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, event: &AuditEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO audit_events (
                id, team_id, task_id, agent_id, event_type,
                severity, description, metadata, user_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5::event_type, $6::event_severity, $7, $8, $9, $10)
            "#,
            event.id,
            event.team_id,
            event.task_id,
            event.agent_id,
            event.event_type.to_string() as _,
            event.severity.to_string() as _,
            event.description,
            event.metadata,
            event.user_id,
            event.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_team(
        &self,
        team_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditEvent>, sqlx::Error> {
        let events = sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT
                id, team_id, task_id, agent_id,
                event_type as "event_type: _",
                severity as "severity: _",
                description, metadata as "metadata: _",
                user_id, created_at
            FROM audit_events
            WHERE team_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            team_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(events)
    }

    pub async fn find_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<AuditEvent>, sqlx::Error> {
        let events = sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT
                id, team_id, task_id, agent_id,
                event_type as "event_type: _",
                severity as "severity: _",
                description, metadata as "metadata: _",
                user_id, created_at
            FROM audit_events
            WHERE task_id = $1
            ORDER BY created_at ASC
            "#,
            task_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(events)
    }

    pub async fn find_by_filters(
        &self,
        team_id: Uuid,
        event_types: Option<Vec<EventType>>,
        severities: Option<Vec<EventSeverity>>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<AuditEvent>, sqlx::Error> {
        // Build dynamic query based on filters
        let mut query = String::from(
            "SELECT id, team_id, task_id, agent_id, event_type, severity, \
             description, metadata, user_id, created_at \
             FROM audit_events WHERE team_id = $1"
        );

        // TODO: Add filter logic (implement with sqlx QueryBuilder)

        let events = sqlx::query_as::<_, AuditEvent>(&query)
            .bind(team_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(events)
    }
}
```

- [ ] 4.1.5: Create audit service

```rust
// apps/api/src/services/audit_service.rs
use crate::domain::audit::event::{AuditEvent, EventType, EventSeverity};
use crate::infrastructure::database::repositories::audit::AuditRepository;
use uuid::Uuid;

pub struct AuditService {
    repository: AuditRepository,
}

impl AuditService {
    pub fn new(repository: AuditRepository) -> Self {
        Self { repository }
    }

    pub async fn log_team_created(&self, team_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        let event = AuditEvent::new(
            team_id,
            EventType::TeamCreated,
            "Team created".to_string(),
        ).with_metadata(serde_json::json!({
            "user_id": user_id
        }));

        self.repository.create(&event).await
    }

    pub async fn log_task_assigned(
        &self,
        team_id: Uuid,
        task_id: Uuid,
        agent_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let event = AuditEvent::new(
            team_id,
            EventType::TaskAssigned,
            "Task assigned to agent".to_string(),
        )
        .with_task(task_id)
        .with_agent(agent_id);

        self.repository.create(&event).await
    }

    pub async fn log_llm_call(
        &self,
        team_id: Uuid,
        task_id: Option<Uuid>,
        model: String,
        tokens: i32,
        cost: f64,
        duration_ms: i64,
    ) -> Result<(), sqlx::Error> {
        let mut event = AuditEvent::new(
            team_id,
            EventType::LlmCallCompleted,
            format!("LLM call completed ({} tokens)", tokens),
        ).with_metadata(serde_json::json!({
            "model": model,
            "tokens": tokens,
            "cost": cost,
            "duration_ms": duration_ms
        }));

        if let Some(tid) = task_id {
            event = event.with_task(tid);
        }

        self.repository.create(&event).await
    }

    pub async fn log_error(
        &self,
        team_id: Uuid,
        task_id: Option<Uuid>,
        error_message: String,
        severity: EventSeverity,
    ) -> Result<(), sqlx::Error> {
        let mut event = AuditEvent::new(
            team_id,
            EventType::ErrorOccurred,
            error_message.clone(),
        )
        .with_severity(severity)
        .with_metadata(serde_json::json!({
            "error": error_message
        }));

        if let Some(tid) = task_id {
            event = event.with_task(tid);
        }

        self.repository.create(&event).await
    }
}
```

**Acceptance Criteria**:

- [ ] Audit events table created
- [ ] All indexes created for performance
- [ ] Can log events of all types
- [ ] Can query events by team
- [ ] Can query events by task
- [ ] Can filter by event type and severity
- [ ] Timestamps accurate

---

## Epic 5: Audit Trail Viewer Component

### Task 5.1: Build Audit Trail UI

**Type**: Frontend
**Dependencies**: Task 4.1 complete

**Subtasks**:

- [ ] 5.1.1: Create audit events API endpoint

```rust
// apps/api/src/api/handlers/audit.rs
use crate::api::auth::jwt::Claims;
use crate::services::audit_service::AuditService;
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AuditQuery {
    event_type: Option<String>,
    severity: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
pub struct AuditEventsResponse {
    events: Vec<AuditEvent>,
    total: i64,
}

pub async fn get_team_audit_trail(
    Path(team_id): Path<Uuid>,
    Query(params): Query<AuditQuery>,
    Extension(claims): Extension<Claims>,
    State(service): State<AuditService>,
) -> Result<Json<AuditEventsResponse>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let events = service
        .get_team_events(team_id, limit, offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuditEventsResponse {
        total: events.len() as i64,
        events,
    }))
}
```

- [ ] 5.1.2: Create audit trail component

```typescript
// apps/frontend/src/components/audit/AuditTrail.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Select } from '@/components/ui/select';
import { format } from 'date-fns';
import { Search, Filter } from 'lucide-react';

interface AuditEvent {
  id: string;
  event_type: string;
  severity: string;
  description: string;
  metadata: Record<string, any>;
  created_at: string;
  task_id?: string;
  agent_id?: string;
}

export function AuditTrail({ teamId }: { teamId: string }) {
  const [searchTerm, setSearchTerm] = useState('');
  const [eventTypeFilter, setEventTypeFilter] = useState<string>('all');
  const [severityFilter, setSeverityFilter] = useState<string>('all');

  const { data, isLoading } = useQuery({
    queryKey: ['audit-trail', teamId, eventTypeFilter, severityFilter],
    queryFn: () => fetchAuditTrail(teamId, {
      event_type: eventTypeFilter === 'all' ? undefined : eventTypeFilter,
      severity: severityFilter === 'all' ? undefined : severityFilter,
    }),
  });

  const filteredEvents = data?.events.filter((event: AuditEvent) =>
    event.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'info': return 'bg-blue-100 text-blue-800';
      case 'warning': return 'bg-yellow-100 text-yellow-800';
      case 'error': return 'bg-red-100 text-red-800';
      case 'critical': return 'bg-red-600 text-white';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getEventIcon = (eventType: string) => {
    // Return appropriate icon based on event type
    return '📋';
  };

  if (isLoading) {
    return <div>Loading audit trail...</div>;
  }

  return (
    <div className="space-y-4">
      {/* Filters */}
      <div className="flex gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            placeholder="Search events..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-10"
          />
        </div>

        <Select
          value={eventTypeFilter}
          onValueChange={setEventTypeFilter}
        >
          <option value="all">All Events</option>
          <option value="task_created">Task Created</option>
          <option value="task_completed">Task Completed</option>
          <option value="llm_call_completed">LLM Calls</option>
          <option value="error_occurred">Errors</option>
        </Select>

        <Select
          value={severityFilter}
          onValueChange={setSeverityFilter}
        >
          <option value="all">All Severities</option>
          <option value="info">Info</option>
          <option value="warning">Warning</option>
          <option value="error">Error</option>
          <option value="critical">Critical</option>
        </Select>
      </div>

      {/* Event List */}
      <div className="space-y-2">
        {filteredEvents?.map((event: AuditEvent) => (
          <Card key={event.id} className="p-4 hover:shadow-md transition-shadow">
            <div className="flex items-start gap-4">
              <div className="text-2xl">{getEventIcon(event.event_type)}</div>

              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <Badge className={getSeverityColor(event.severity)}>
                    {event.severity}
                  </Badge>
                  <span className="text-sm text-gray-500">
                    {format(new Date(event.created_at), 'MMM d, yyyy HH:mm:ss')}
                  </span>
                </div>

                <p className="font-medium mb-1">{event.description}</p>

                {Object.keys(event.metadata).length > 0 && (
                  <details className="text-sm text-gray-600">
                    <summary className="cursor-pointer hover:text-gray-900">
                      View metadata
                    </summary>
                    <pre className="mt-2 p-2 bg-gray-50 rounded text-xs overflow-x-auto">
                      {JSON.stringify(event.metadata, null, 2)}
                    </pre>
                  </details>
                )}
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Pagination */}
      {filteredEvents && filteredEvents.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          No events found matching your filters
        </div>
      )}
    </div>
  );
}
```

- [ ] 5.1.3: Create task-specific audit view

```typescript
// apps/frontend/src/components/audit/TaskAuditHistory.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { Card } from '@/components/ui/card';
import { format } from 'date-fns';

export function TaskAuditHistory({ taskId }: { taskId: string }) {
  const { data: events } = useQuery({
    queryKey: ['task-audit', taskId],
    queryFn: () => fetchTaskAuditHistory(taskId),
  });

  return (
    <div className="relative">
      {/* Timeline */}
      <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-gray-200" />

      <div className="space-y-4">
        {events?.map((event, index) => (
          <div key={event.id} className="relative pl-10">
            {/* Timeline dot */}
            <div className="absolute left-2.5 top-2 w-3 h-3 rounded-full bg-blue-500 ring-4 ring-white" />

            <Card className="p-3">
              <div className="flex justify-between items-start mb-1">
                <span className="font-medium text-sm">{event.description}</span>
                <span className="text-xs text-gray-500">
                  {format(new Date(event.created_at), 'HH:mm:ss')}
                </span>
              </div>

              {event.metadata && (
                <div className="text-xs text-gray-600">
                  {Object.entries(event.metadata).map(([key, value]) => (
                    <div key={key}>
                      <span className="font-medium">{key}:</span> {String(value)}
                    </div>
                  ))}
                </div>
              )}
            </Card>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] Can view all audit events for a team
- [ ] Can filter by event type
- [ ] Can filter by severity
- [ ] Can search event descriptions
- [ ] Can view event metadata
- [ ] Timeline view for task history
- [ ] Pagination works correctly

---

## Epic 6: Message History Timeline

### Task 6.1: Message History Viewer

**Type**: Frontend
**Dependencies**: Messages table exists

**Subtasks**:

- [ ] 6.1.1: Create messages API endpoint

```rust
// apps/api/src/api/handlers/messages.rs
use crate::services::message_service::MessageService;
use axum::{extract::{Path, Query, State}, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct MessageQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    message_type: Option<String>,
}

pub async fn get_team_messages(
    Path(team_id): Path<Uuid>,
    Query(params): Query<MessageQuery>,
    State(service): State<MessageService>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    let messages = service
        .get_team_messages(
            team_id,
            params.limit.unwrap_or(100),
            params.offset.unwrap_or(0),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}
```

- [ ] 6.1.2: Create message timeline component

```typescript
// apps/frontend/src/components/messages/MessageTimeline.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { useTeamUpdates } from '@/hooks/useTeamUpdates';
import { Card } from '@/components/ui/card';
import { Avatar } from '@/components/ui/avatar';
import { Badge } from '@/components/ui/badge';
import { format } from 'date-fns';
import { motion, AnimatePresence } from 'framer-motion';

interface Message {
  id: string;
  from_agent_id: string;
  to_agent_id?: string;
  message_type: string;
  content: string;
  metadata: Record<string, any>;
  created_at: string;
}

export function MessageTimeline({ teamId }: { teamId: string }) {
  useTeamUpdates(teamId);

  const { data: messages, isLoading } = useQuery({
    queryKey: ['messages', teamId],
    queryFn: () => fetchTeamMessages(teamId),
    refetchInterval: false, // Only update via WebSocket
  });

  const getMessageTypeColor = (type: string) => {
    switch (type) {
      case 'task_assignment': return 'bg-blue-100 text-blue-800';
      case 'task_completion': return 'bg-green-100 text-green-800';
      case 'revision_request': return 'bg-yellow-100 text-yellow-800';
      case 'approval': return 'bg-green-100 text-green-800';
      case 'rejection': return 'bg-red-100 text-red-800';
      case 'agent_communication': return 'bg-purple-100 text-purple-800';
      case 'system_event': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (isLoading) {
    return <div>Loading messages...</div>;
  }

  return (
    <div className="space-y-4">
      <AnimatePresence>
        {messages?.map((message: Message, index: number) => (
          <motion.div
            key={message.id}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 20 }}
            transition={{ delay: index * 0.05 }}
          >
            <Card className="p-4">
              <div className="flex gap-4">
                <Avatar className="w-10 h-10">
                  <div className="w-full h-full bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white text-sm font-bold">
                    A
                  </div>
                </Avatar>

                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="font-semibold">
                      Agent {message.from_agent_id.slice(0, 8)}
                    </span>
                    {message.to_agent_id && (
                      <>
                        <span className="text-gray-400">→</span>
                        <span className="text-gray-600">
                          Agent {message.to_agent_id.slice(0, 8)}
                        </span>
                      </>
                    )}
                    <Badge className={getMessageTypeColor(message.message_type)}>
                      {message.message_type.replace('_', ' ')}
                    </Badge>
                    <span className="text-sm text-gray-500 ml-auto">
                      {format(new Date(message.created_at), 'MMM d, HH:mm')}
                    </span>
                  </div>

                  <p className="text-gray-700 whitespace-pre-wrap">
                    {message.content}
                  </p>

                  {message.metadata && Object.keys(message.metadata).length > 0 && (
                    <div className="mt-2 text-xs text-gray-500">
                      {Object.entries(message.metadata).map(([key, value]) => (
                        <span key={key} className="mr-3">
                          {key}: {String(value)}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </Card>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
```

- [ ] 6.1.3: Add message type filter

```typescript
// apps/frontend/src/components/messages/MessageFilter.tsx
'use client';

import { Select } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';

interface MessageFilterProps {
  selectedTypes: string[];
  onTypesChange: (types: string[]) => void;
}

const MESSAGE_TYPES = [
  { value: 'task_assignment', label: 'Task Assignments' },
  { value: 'task_completion', label: 'Completions' },
  { value: 'revision_request', label: 'Revisions' },
  { value: 'approval', label: 'Approvals' },
  { value: 'rejection', label: 'Rejections' },
  { value: 'agent_communication', label: 'Agent Chat' },
  { value: 'system_event', label: 'System Events' },
];

export function MessageFilter({ selectedTypes, onTypesChange }: MessageFilterProps) {
  const toggleType = (type: string) => {
    if (selectedTypes.includes(type)) {
      onTypesChange(selectedTypes.filter(t => t !== type));
    } else {
      onTypesChange([...selectedTypes, type]);
    }
  };

  return (
    <div className="flex flex-wrap gap-2">
      {MESSAGE_TYPES.map(({ value, label }) => (
        <Badge
          key={value}
          variant={selectedTypes.includes(value) ? 'default' : 'outline'}
          className="cursor-pointer"
          onClick={() => toggleType(value)}
        >
          {label}
        </Badge>
      ))}
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] Can view all team messages
- [ ] Messages display in chronological order
- [ ] Can filter by message type
- [ ] Real-time updates via WebSocket
- [ ] Smooth animations for new messages
- [ ] Agent avatars display correctly

---

## Success Criteria - Phase 6 Complete

- [ ] WebSocket server running and stable
- [ ] Clients can connect and subscribe to teams
- [ ] Real-time updates broadcasting correctly
- [ ] Audit events logging all important actions
- [ ] Audit trail viewer functional and filterable
- [ ] Message history displays correctly
- [ ] Task timeline shows complete history
- [ ] No WebSocket connection leaks
- [ ] Reconnection logic working
- [ ] Performance acceptable with 100+ concurrent connections

---

## Next Steps

Proceed to [10-phase-7-error-recovery.md](./10-phase-7-error-recovery.md) for checkpoint and failure recovery implementation.

---

**Phase 6: Real-time and audit systems operational**
