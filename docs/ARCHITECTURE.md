## 📐 BL1NK Architecture - Rust + Next.js 16.1.6

## 🏗️ Architecture Overview

````artifact
id: blink_rust_nextjs_arch
name: BL1NK Rust + Next.js Architecture Diagram
type: svg
content: |-
  <svg viewBox="0 0 1200 800" xmlns="http://www.w3.org/2000/svg">
    <!-- Title -->
    <text x="600" y="30" font-size="28" font-weight="bold" text-anchor="middle" fill="#1a1a1a">
      BL1NK: Rust Backend + Next.js 16.1.6 Frontend
    </text>
    
    <!-- Frontend Layer -->
    <rect x="50" y="80" width="500" height="280" fill="#e8f4f8" stroke="#0284c7" stroke-width="2" rx="8"/>
    <text x="300" y="110" font-size="18" font-weight="bold" text-anchor="middle" fill="#0284c7">
      Frontend Layer - Next.js 16.1.6
    </text>
    
    <!-- Frontend Components -->
    <rect x="70" y="140" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="140" y="165" font-size="12" font-weight="bold" text-anchor="middle">Dashboard</text>
    <text x="140" y="180" font-size="10" text-anchor="middle" fill="#666">(/ route)</text>
    
    <rect x="230" y="140" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="300" y="165" font-size="12" font-weight="bold" text-anchor="middle">Sessions</text>
    <text x="300" y="180" font-size="10" text-anchor="middle" fill="#666">(/sessions)</text>
    
    <rect x="390" y="140" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="460" y="165" font-size="12" font-weight="bold" text-anchor="middle">Config</text>
    <text x="460" y="180" font-size="10" text-anchor="middle" fill="#666">(/config)</text>
    
    <rect x="70" y="230" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="140" y="255" font-size="12" font-weight="bold" text-anchor="middle">Monitor</text>
    <text x="140" y="270" font-size="10" text-anchor="middle" fill="#666">Real-time</text>
    
    <rect x="230" y="230" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="300" y="255" font-size="12" font-weight="bold" text-anchor="middle">WebSocket</text>
    <text x="300" y="270" font-size="10" text-anchor="middle" fill="#666">Stream</text>
    
    <rect x="390" y="230" width="140" height="60" fill="#ffffff" stroke="#0284c7" stroke-width="1.5" rx="4"/>
    <text x="460" y="255" font-size="12" font-weight="bold" text-anchor="middle">State Mgmt</text>
    <text x="460" y="270" font-size="10" text-anchor="middle" fill="#666">Context</text>
    
    <!-- Backend Layer -->
    <rect x="650" y="80" width="500" height="280" fill="#fef3c7" stroke="#d97706" stroke-width="2" rx="8"/>
    <text x="900" y="110" font-size="18" font-weight="bold" text-anchor="middle" fill="#d97706">
      Backend Layer - Rust + Axum
    </text>
    
    <!-- Backend Components -->
    <rect x="670" y="140" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="735" y="165" font-size="12" font-weight="bold" text-anchor="middle">Execution</text>
    <text x="735" y="180" font-size="10" text-anchor="middle" fill="#666">Orchestrator</text>
    
    <rect x="820" y="140" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="885" y="165" font-size="12" font-weight="bold" text-anchor="middle">Provider</text>
    <text x="885" y="180" font-size="10" text-anchor="middle" fill="#666">Service</text>
    
    <rect x="970" y="140" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="1035" y="165" font-size="12" font-weight="bold" text-anchor="middle">Git</text>
    <text x="1035" y="180" font-size="10" text-anchor="middle" fill="#666">Service</text>
    
    <rect x="670" y="230" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="735" y="255" font-size="12" font-weight="bold" text-anchor="middle">Config</text>
    <text x="735" y="270" font-size="10" text-anchor="middle" fill="#666">Service</text>
    
    <rect x="820" y="230" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="885" y="255" font-size="12" font-weight="bold" text-anchor="middle">State</text>
    <text x="885" y="270" font-size="10" text-anchor="middle" fill="#666">Manager</text>
    
    <rect x="970" y="230" width="130" height="60" fill="#ffffff" stroke="#d97706" stroke-width="1.5" rx="4"/>
    <text x="1035" y="255" font-size="12" font-weight="bold" text-anchor="middle">Database</text>
    <text x="1035" y="270" font-size="10" text-anchor="middle" fill="#666">SQLite/Postgres</text>
    
    <!-- Communication Layer -->
    <rect x="50" y="420" width="1100" height="120" fill="#f0fdf4" stroke="#16a34a" stroke-width="2" rx="8"/>
    <text x="600" y="450" font-size="18" font-weight="bold" text-anchor="middle" fill="#16a34a">
      Communication Layer
    </text>
    
    <rect x="100" y="470" width="200" height="50" fill="#ffffff" stroke="#16a34a" stroke-width="1.5" rx="4"/>
    <text x="200" y="500" font-size="12" font-weight="bold" text-anchor="middle">REST API</text>
    
    <rect x="350" y="470" width="200" height="50" fill="#ffffff" stroke="#16a34a" stroke-width="1.5" rx="4"/>
    <text x="450" y="500" font-size="12" font-weight="bold" text-anchor="middle">WebSocket</text>
    
    <rect x="600" y="470" width="200" height="50" fill="#ffffff" stroke="#16a34a" stroke-width="1.5" rx="4"/>
    <text x="700" y="500" font-size="12" font-weight="bold" text-anchor="middle">JSON-RPC</text>
    
    <rect x="850" y="470" width="200" height="50" fill="#ffffff" stroke="#16a34a" stroke-width="1.5" rx="4"/>
    <text x="950" y="500" font-size="12" font-weight="bold" text-anchor="middle">gRPC (Optional)</text>
    
    <!-- Data Flow -->
    <rect x="50" y="600" width="1100" height="150" fill="#faf5ff" stroke="#9333ea" stroke-width="2" rx="8"/>
    <text x="600" y="630" font-size="18" font-weight="bold" text-anchor="middle" fill="#9333ea">
      Data Flow & State Management
    </text>
    
    <!-- Flow boxes -->
    <circle cx="150" cy="700" r="35" fill="#ffffff" stroke="#9333ea" stroke-width="2"/>
    <text x="150" y="705" font-size="11" font-weight="bold" text-anchor="middle">Request</text>
    
    <circle cx="300" cy="700" r="35" fill="#ffffff" stroke="#9333ea" stroke-width="2"/>
    <text x="300" y="705" font-size="11" font-weight="bold" text-anchor="middle">Process</text>
    
    <circle cx="450" cy="700" r="35" fill="#ffffff" stroke="#9333ea" stroke-width="2"/>
    <text x="450" y="705" font-size="11" font-weight="bold" text-anchor="middle">Persist</text>
    
    <circle cx="600" cy="700" r="35" fill="#ffffff" stroke="#9333ea" stroke-width="2"/>
    <text x="600" y="705" font-size="11" font-weight="bold" text-anchor="middle">Stream</text>
    
    <circle cx="750" cy="700" r="35" fill="#ffffff" stroke="#9333ea" stroke-width="2"/>
    <text x="750" y="705" font-size="11" font-weight="bold" text-anchor="middle">Render</text>
    
    <!-- Arrows -->
    <path d="M 185 700 L 265 700" stroke="#9333ea" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>
    <path d="M 335 700 L 415 700" stroke="#9333ea" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>
    <path d="M 485 700 L 565 700" stroke="#9333ea" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>
    <path d="M 635 700 L 715 700" stroke="#9333ea" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>
    
    <!-- Arrow marker definition -->
    <defs>
      <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
        <polygon points="0 0, 10 3, 0 6" fill="#9333ea" />
      </marker>
    </defs>
  </svg>
````

---

## 📁 Directory Structure

```
blink-nexus/
├── backend/                          # Rust Backend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                   # Server entry point
│   │   ├── lib.rs
│   │   ├── handlers/                 # HTTP handlers
│   │   │   ├── mod.rs
│   │   │   ├── execution.rs          # /api/execute
│   │   │   ├── sessions.rs           # /api/sessions
│   │   │   ├── config.rs             # /api/config
│   │   │   └── ws.rs                 # WebSocket handler
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── execution/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sequential.rs     # Sequential orchestrator
│   │   │   │   ├── task_source.rs    # State machine
│   │   │   │   └── prompt.rs         # Prompt generation
│   │   │   ├── provider/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── base.rs           # Trait definition
│   │   │   │   ├── gemini.rs         # Gemini provider
│   │   │   │   ├── opencode.rs       # Opencode provider
│   │   │   │   └── factory.rs        # Provider factory
│   │   │   ├── git/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── worktree.rs       # Sandbox management
│   │   │   │   ├── diff.rs           # Diff analysis
│   │   │   │   ├── branch.rs         # Branch operations
│   │   │   │   └── pr.rs             # PR automation
│   │   │   ├── config/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── settings.rs       # User settings
│   │   │   │   ├── state.rs          # State persistence
│   │   │   │   └── types.rs          # Zod-like schemas
│   │   │   └── database/
│   │   │       ├── mod.rs
│   │   │       ├── models.rs         # Data models
│   │   │       └── queries.rs        # DB queries
│   │   ├── models/                   # Data structures
│   │   │   ├── mod.rs
│   │   │   ├── execution.rs
│   │   │   ├── session.rs
│   │   │   ├── provider.rs
│   │   │   └── git.rs
│   │   ├── error.rs                  # Error handling
│   │   ├── config.rs                 # Server config
│   │   └── utils.rs                  # Utility functions
│   └── tests/
│
├── frontend/                         # Next.js 16.1.6
│   ├── package.json
│   ├── next.config.js
│   ├── tsconfig.json
│   ├── app/
│   │   ├── layout.tsx                # Root layout
│   │   ├── page.tsx                  # Dashboard (/)
│   │   ├── sessions/
│   │   │   ├── page.tsx              # Sessions list
│   │   │   └── [id]/
│   │   │       └── page.tsx          # Session detail
│   │   ├── config/
│   │   │   └── page.tsx              # Configuration
│   │   ├── execute/
│   │   │   └── page.tsx              # Execution monitor
│   │   └── api/
│   │       └── ws/
│   │           └── route.ts          # WebSocket route
│   ├── components/
│   │   ├── Dashboard/
│   │   │   ├── Nexus.tsx             # Main dashboard
│   │   │   ├── LandingView.tsx       # Initial prompt
│   │   │   └── ProgressMonitor.tsx   # Progress tracker
│   │   ├── Sessions/
│   │   │   ├── SessionList.tsx
│   │   │   ├── SessionDetail.tsx
│   │   │   └── SessionCard.tsx
│   │   ├── Config/
│   │   │   ├── ProviderConfig.tsx
│   │   │   ├── SettingsForm.tsx
│   │   │   └── ModelSelector.tsx
│   │   ├── Common/
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Spinner.tsx
│   │   │   └── ErrorBoundary.tsx
│   │   └── Stream/
│   │       ├── StreamViewer.tsx      # Real-time output
│   │       └── JSONLineParser.tsx
│   ├── hooks/
│   │   ├── useExecution.ts           # Execution logic
│   │   ├── useWebSocket.ts           # WebSocket connection
│   │   ├── useSessions.ts            # Sessions management
│   │   └── useConfig.ts              # Config management
│   ├── lib/
│   │   ├── api.ts                    # API client
│   │   ├── ws.ts                     # WebSocket client
│   │   ├── types.ts                  # TypeScript types
│   │   └── utils.ts                  # Utilities
│   ├── styles/
│   │   └── globals.css
│   └── public/
│
└── docker-compose.yml                # Docker setup
```

---

## 🦀 Rust Backend - Key Files

### **1. `src/main.rs`** - Server Entry Point

````rust
use axum::{
    routing::{get, post},
    Router, extract::ws::WebSocketUpgrade,
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod handlers;
mod services;
mod models;
mod error;
mod config;

#[tokio::main]
async fn main() {
    let config = config::load_config();
    
    let app = Router::new()
        // REST API Routes
        .route("/api/execute", post(handlers::execution::execute))
        .route("/api/sessions", get(handlers::sessions::list_sessions))
        .route("/api/sessions/:id", get(handlers::sessions::get_session))
        .route("/api/config", get(handlers::config::get_config))
        .route("/api/config", post(handlers::config::update_config))
        
        // WebSocket Route
        .route("/ws", get(handlers::ws::ws_handler))
        
        // Health check
        .route("/health", get(|| async { "OK" }))
        
        .layer(CorsLayer::permissive());
    
    let listener = TcpListener::bind("127.0.0.1:3001")
        .await
        .expect("Failed to bind");
    
    println!("🚀 BL1NK Server running on http://127.0.0.1:3001");
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
````

### **2. `src/services/execution/sequential.rs`** - Orchestrator

````rust
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPhase {
    Brief,
    Decompose,
    Investigate,
    Design,
    Execute,
    Optimize,
    Complete,
}

pub struct SequentialExecutor {
    state: ExecutionState,
    provider: Box<dyn AIProvider>,
    tx: mpsc::UnboundedSender<ProgressUpdate>,
}

#[derive(Debug, Serialize)]
pub struct ProgressUpdate {
    pub phase: ExecutionPhase,
    pub message: String,
    pub percentage: u32,
}

impl SequentialExecutor {
    pub async fn execute(&mut self) -> Result<ExecutionResult, ExecutionError> {
        let phases = vec![
            ExecutionPhase::Brief,
            ExecutionPhase::Decompose,
            ExecutionPhase::Investigate,
            ExecutionPhase::Design,
            ExecutionPhase::Execute,
            ExecutionPhase::Optimize,
            ExecutionPhase::Complete,
        ];
        
        for (idx, phase) in phases.iter().enumerate() {
            self.execute_phase(phase).await?;
            
            let progress = ProgressUpdate {
                phase: phase.clone(),
                message: format!("Completed: {:?}", phase),
                percentage: ((idx + 1) * 100 / phases.len()) as u32,
            };
            
            let _ = self.tx.send(progress);
        }
        
        Ok(ExecutionResult::success())
    }
    
    async fn execute_phase(&mut self, phase: &ExecutionPhase) -> Result<(), ExecutionError> {
        let prompt = self.generate_prompt(phase)?;
        
        // Stream response from provider
        let mut stream = self.provider.execute_stream(&prompt).await?;
        
        while let Some(chunk) = stream.next().await {
            // Process streaming JSON
            self.process_chunk(&chunk)?;
        }
        
        Ok(())
    }
}
````

### **3. `src/services/provider/base.rs`** - Provider Trait

````rust
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn execute(&self, prompt: &str) -> Result<String, ProviderError>;
    
    async fn execute_stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn futures::Stream<Item = String> + Unpin>, ProviderError>;
    
    fn name(&self) -> &str;
}

pub struct GeminiProvider {
    api_key: String,
    model: String,
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn execute(&self, prompt: &str) -> Result<String, ProviderError> {
        // Call Gemini CLI
        let output = tokio::process::Command::new("gemini")
            .arg("--prompt")
            .arg(prompt)
            .output()
            .await?;
        
        Ok(String::from_utf8(output.stdout)?)
    }
    
    async fn execute_stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn futures::Stream<Item = String> + Unpin>, ProviderError> {
        // Streaming implementation
        todo!()
    }
    
    fn name(&self) -> &str {
        "gemini"
    }
}
````

### **4. `src/handlers/ws.rs`** - WebSocket Handler

````rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to execution updates
    let mut rx = get_execution_channel(); // From global state
    
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            let _ = sender.send(Message::Text(json)).await;
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                // Handle client commands
                println!("Received: {}", text);
            }
            _ => {}
        }
    }
}
````

---

## ⚛️ Next.js Frontend - Key Files

### **1. `app/page.tsx`** - Dashboard

````typescript
'use client';

import React, { useState } from 'react';
import Dashboard from '@/components/Dashboard/Nexus';
import LandingView from '@/components/Dashboard/LandingView';

export default function Home() {
  const [sessionActive, setSessionActive] = useState(false);
  
  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-900 to-slate-800">
      {!sessionActive ? (
        <LandingView onStart={() => setSessionActive(true)} />
      ) : (
        <Dashboard />
      )}
    </main>
  );
}
````

### **2. `components/Dashboard/Nexus.tsx`** - Main Dashboard

````typescript
'use client';

import React, { useEffect, useState } from 'react';
import { useExecution } from '@/hooks/useExecution';
import ProgressMonitor from './ProgressMonitor';
import StreamViewer from '@/components/Stream/StreamViewer';

export default function Dashboard() {
  const { execute, progress, stream, isRunning } = useExecution();
  const [prompt, setPrompt] = useState('');
  
  const handleExecute = async () => {
    await execute(prompt);
  };
  
  return (
    <div className="p-8 space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-2">
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Enter your task..."
            className="w-full h-32 p-4 bg-slate-800 text-white rounded-lg border border-slate-700"
            disabled={isRunning}
          />
          <button
            onClick={handleExecute}
            disabled={isRunning}
            className="mt-4 px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50"
          >
            {isRunning ? 'Executing...' : 'Execute'}
          </button>
        </div>
        
        <ProgressMonitor progress={progress} />
      </div>
      
      <StreamViewer stream={stream} />
    </div>
  );
}
````

### **3. `hooks/useExecution.ts`** - Execution Hook

````typescript
'use client';

import { useState, useCallback, useEffect } from 'react';
import { useWebSocket } from './useWebSocket';
import { api } from '@/lib/api';

interface ProgressUpdate {
  phase: string;
  message: string;
  percentage: number;
}

export function useExecution() {
  const [progress, setProgress] = useState<ProgressUpdate | null>(null);
  const [stream, setStream] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  
  const { send, subscribe } = useWebSocket('ws://localhost:3001/ws');
  
  useEffect(() => {
    subscribe('progress', (data: ProgressUpdate) => {
      setProgress(data);
    });
    
    subscribe('stream', (data: string) => {
      setStream(prev => [...prev, data]);
    });
  }, [subscribe]);
  
  const execute = useCallback(async (prompt: string) => {
    setIsRunning(true);
    setStream([]);
    
    try {
      const response = await api.post('/api/execute', { prompt });
      const sessionId = response.data.sessionId;
      
      send({
        type: 'subscribe',
        sessionId,
      });
    } catch (error) {
      console.error('Execution error:', error);
    } finally {
      setIsRunning(false);
    }
  }, [send]);
  
  return { execute, progress, stream, isRunning };
}
````

### **4. `hooks/useWebSocket.ts`** - WebSocket Hook

````typescript
'use client';

import { useEffect, useRef, useCallback } from 'react';

export function useWebSocket(url: string) {
  const ws = useRef<WebSocket | null>(null);
  const handlers = useRef<Map<string, Function>>(new Map());
  
  useEffect(() => {
    ws.current = new WebSocket(url);
    
    ws.current.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        const handler = handlers.current.get(data.type);
        if (handler) {
          handler(data.payload);
        }
      } catch (error) {
        console.error('WebSocket message error:', error);
      }
    };
    
    return () => {
      ws.current?.close();
    };
  }, [url]);
  
  const send = useCallback((data: any) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(data));
    }
  }, []);
  
  const subscribe = useCallback((type: string, handler: Function) => {
    handlers.current.set(type, handler);
  }, []);
  
  return { send, subscribe };
}
````

---

## 📦 Cargo.toml - Dependencies

````toml
[package]
name = "blink-nexus"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web Framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Async
futures = "0.3"
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# CLI
clap = { version = "4.4", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Git operations
git2 = "0.18"

# Process execution
tokio-util = "0.7"

# Testing
tokio-test = "0.4"
````

---

## 🚀 Running the Application

### **Backend (Rust)**

```bash
cd backend
cargo build --release
cargo run
```

### **Frontend (Next.js)**

```bash
cd frontend
npm install
npm run dev
```

### **Docker Compose**

````yaml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "3001:3001"
    environment:
      - RUST_LOG=debug
    volumes:
      - ./backend:/app

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    depends_on:
      - backend
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:3001

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=blink
      - POSTGRES_PASSWORD=blink_dev
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
````

---

## 🔄 Data Flow Summary

```
┌─────────────────────────────────────────────────────────┐
│  User Input (Next.js Dashboard)                         │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  REST API Request → Rust Backend (Axum)                │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Sequential Executor (Multi-phase orchestration)       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  AI Provider Service (Gemini/Opencode)                 │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Git Service (Worktree Sandbox Operations)             │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  State Persistence (SQLite/Postgres)                   │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  WebSocket Stream → Next.js Frontend                   │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│  Real-time UI Updates (React Components)               │
└─────────────────────────────────────────────────────────┘
```