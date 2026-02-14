---

## 🦀 BL1NK Autonomous Execution Engine - Rust Architecture

### Overview

**BL1NK Rust Edition** เป็น high-performance coding orchestrator ที่เขียนด้วย Rust พร้อม TUI interface ที่ใช้ `ratatui` และ async runtime ที่ใช้ `Tokio`

---

## Core Components Architecture

````artifact
id: blink_rust_architecture
name: BL1NK Rust Architecture Diagram
type: html
content: |-
  <!DOCTYPE html>
  <html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BL1NK Rust Architecture</title>
    <style>
      body {
        font-family: 'Courier New', monospace;
        background: #0d1117;
        color: #c9d1d9;
        padding: 20px;
        margin: 0;
      }
      .container {
        max-width: 1200px;
        margin: 0 auto;
      }
      .title {
        text-align: center;
        color: #58a6ff;
        margin-bottom: 30px;
        font-size: 24px;
        font-weight: bold;
      }
      .layer {
        background: #161b22;
        border: 2px solid #30363d;
        border-radius: 8px;
        padding: 15px;
        margin-bottom: 20px;
        box-shadow: 0 4px 6px rgba(0,0,0,0.3);
      }
      .layer-title {
        color: #79c0ff;
        font-weight: bold;
        margin-bottom: 10px;
        font-size: 14px;
        text-transform: uppercase;
      }
      .component {
        background: #0d1117;
        border-left: 3px solid #58a6ff;
        padding: 10px 15px;
        margin: 8px 0;
        border-radius: 4px;
      }
      .component-name {
        color: #79c0ff;
        font-weight: bold;
      }
      .component-desc {
        color: #8b949e;
        font-size: 12px;
        margin-top: 4px;
      }
      .arrow {
        text-align: center;
        color: #58a6ff;
        font-size: 18px;
        margin: 10px 0;
      }
      .grid {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr;
        gap: 15px;
        margin-top: 15px;
      }
      .grid-item {
        background: #0d1117;
        border: 1px solid #30363d;
        padding: 12px;
        border-radius: 4px;
      }
      .grid-item-title {
        color: #79c0ff;
        font-weight: bold;
        margin-bottom: 8px;
      }
      .code {
        background: #010409;
        border: 1px solid #30363d;
        padding: 10px;
        border-radius: 4px;
        font-size: 11px;
        color: #79c0ff;
        margin-top: 8px;
        overflow-x: auto;
      }
    </style>
  </head>
  <body>
    <div class="container">
      <div class="title">🦀 BL1NK Rust Architecture</div>
      
      <div class="layer">
        <div class="layer-title">📍 Entry Point: src/main.rs</div>
        <div class="component">
          <div class="component-name">BL1NK CLI Gateway</div>
          <div class="component-desc">Clap CLI Parser + Tokio Async Runtime</div>
          <div class="code">use clap::{Parser, Subcommand};<br>use tokio::runtime::Runtime;</div>
        </div>
      </div>

      <div class="arrow">↓</div>

      <div class="grid">
        <div class="grid-item">
          <div class="grid-item-title">🎨 TUI Mode</div>
          <div class="component-desc">ratatui Dashboard</div>
          <div class="code">src/ui/dashboard.rs</div>
        </div>
        <div class="grid-item">
          <div class="grid-item-title">⚙️ Headless Mode</div>
          <div class="component-desc">Direct Execution</div>
          <div class="code">src/executor.rs</div>
        </div>
        <div class="grid-item">
          <div class="grid-item-title">📋 Session Mode</div>
          <div class="component-desc">Resume & History</div>
          <div class="code">src/session.rs</div>
        </div>
      </div>

      <div class="arrow">↓</div>

      <div class="layer">
        <div class="layer-title">🔄 Sequential Orchestrator</div>
        <div class="component">
          <div class="component-name">SequentialExecutor</div>
          <div class="component-desc">Multi-phase coordination with state machine</div>
          <div class="code">src/services/execution/executor.rs</div>
        </div>
      </div>

      <div class="arrow">↓</div>

      <div class="grid">
        <div class="grid-item">
          <div class="grid-item-title">📦 Task Pipeline</div>
          <div class="code">src/services/task.rs</div>
        </div>
        <div class="grid-item">
          <div class="grid-item-title">🤖 AI Providers</div>
          <div class="code">src/services/providers/</div>
        </div>
        <div class="grid-item">
          <div class="grid-item-title">🌳 Git Sandbox</div>
          <div class="code">src/services/git/</div>
        </div>
      </div>

      <div class="layer" style="margin-top: 30px;">
        <div class="layer-title">📁 Directory Structure</div>
        <div class="code">
src/<br>
├── main.rs                    # CLI Gateway<br>
├── lib.rs                     # Public API<br>
├── executor.rs               # Core Executor<br>
├── session.rs                # Session Management<br>
├── state.rs                  # State Machine<br>
├── services/<br>
│   ├── execution/<br>
│   │   ├── mod.rs<br>
│   │   ├── executor.rs       # SequentialExecutor<br>
│   │   ├── phases.rs         # Execution Phases<br>
│   │   └── prompt.rs         # Prompt Generation<br>
│   ├── providers/<br>
│   │   ├── mod.rs<br>
│   │   ├── base.rs           # Provider Trait<br>
│   │   ├── gemini.rs         # Gemini Provider<br>
│   │   └── factory.rs        # Provider Factory<br>
│   ├── git/<br>
│   │   ├── mod.rs<br>
│   │   ├── worktree.rs       # Sandbox Management<br>
│   │   ├── diff.rs           # Diff Analysis<br>
│   │   └── branch.rs         # Branch Operations<br>
│   └── config/<br>
│       ├── mod.rs<br>
│       ├── settings.rs       # User Settings<br>
│       └── types.rs          # Zod-like Validation<br>
├── ui/<br>
│   ├── mod.rs<br>
│   ├── dashboard.rs          # Main Dashboard<br>
│   ├── components.rs         # UI Components<br>
│   └── events.rs             # Event Handling<br>
└── utils.rs                  # Utilities
        </div>
      </div>
    </div>
  </body>
  </html>
````

---

## 📋 Key Rust Components

### 1. **CLI Gateway** (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(name = "BL1NK")]
#[command(about = "Autonomous Execution Engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Interactive TUI mode (default)
    #[arg(short, long)]
    tui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute autonomous agent
    Execute {
        #[arg(short, long)]
        prompt: Option<String>,
        
        #[arg(short, long)]
        resume: Option<String>,
    },
    /// Manage sessions
    Sessions {
        #[arg(short, long)]
        list: bool,
    },
    /// Configure BL1NK
    Config {
        #[arg(short, long)]
        set: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // Implementation
}
```

### 2. **State Machine** (`src/state.rs`)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    pub session_id: String,
    pub phase: ExecutionPhase,
    pub prompt: String,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExecutionState {
    pub fn next_phase(&mut self) -> Option<ExecutionPhase> {
        self.phase = match self.phase {
            ExecutionPhase::Brief => ExecutionPhase::Decompose,
            ExecutionPhase::Decompose => ExecutionPhase::Investigate,
            ExecutionPhase::Investigate => ExecutionPhase::Design,
            ExecutionPhase::Design => ExecutionPhase::Execute,
            ExecutionPhase::Execute => ExecutionPhase::Optimize,
            ExecutionPhase::Optimize => ExecutionPhase::Complete,
            ExecutionPhase::Complete => return None,
        };
        Some(self.phase.clone())
    }
}
```

### 3. **Provider Trait** (`src/services/providers/base.rs`)

```rust
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn execute(
        &self,
        prompt: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<String>;
    
    fn name(&self) -> &str;
}

pub struct ProviderConfig {
    pub provider_type: String,
    pub api_key: String,
    pub model: String,
}
```

### 4. **Sequential Executor** (`src/services/execution/executor.rs`)

```rust
use crate::state::ExecutionState;
use crate::services::providers::Provider;
use tokio::sync::mpsc;

pub struct SequentialExecutor {
    state: ExecutionState,
    provider: Box<dyn Provider>,
}

impl SequentialExecutor {
    pub async fn execute(&mut self) -> Result<()> {
        loop {
            match self.state.phase {
                ExecutionPhase::Brief => self.execute_brief().await?,
                ExecutionPhase::Decompose => self.execute_decompose().await?,
                ExecutionPhase::Investigate => self.execute_investigate().await?,
                ExecutionPhase::Design => self.execute_design().await?,
                ExecutionPhase::Execute => self.execute_execute().await?,
                ExecutionPhase::Optimize => self.execute_optimize().await?,
                ExecutionPhase::Complete => break,
            }
            
            if self.state.next_phase().is_none() {
                break;
            }
        }
        Ok(())
    }
    
    async fn execute_brief(&mut self) -> Result<()> {
        let prompt = format!("Generate PRD for: {}", self.state.prompt);
        let (tx, mut rx) = mpsc::channel(100);
        
        tokio::spawn({
            let provider = self.provider.clone();
            async move {
                let _ = provider.execute(&prompt, tx).await;
            }
        });
        
        while let Some(chunk) = rx.recv().await {
            self.state.context["brief"] = serde_json::json!(chunk);
        }
        
        Ok(())
    }
}
```

### 5. **Git Worktree Manager** (`src/services/git/worktree.rs`)

```rust
use std::path::PathBuf;
use git2::{Repository, Oid};

pub struct GitSandbox {
    repo_path: PathBuf,
    session_id: String,
    sandbox_path: PathBuf,
}

impl GitSandbox {
    pub fn new(repo_path: PathBuf, session_id: String) -> Result<Self> {
        let sandbox_path = repo_path
            .join(".blink")
            .join("sandboxes")
            .join(&session_id);
        
        Ok(Self {
            repo_path,
            session_id,
            sandbox_path,
        })
    }
    
    pub fn create_worktree(&self, branch: &str) -> Result<()> {
        let repo = Repository::open(&self.repo_path)?;
        
        // Create worktree
        let mut worktree_builder = repo.worktree(branch, &self.sandbox_path, None)?;
        
        Ok(())
    }
    
    pub fn cleanup(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.sandbox_path)?;
        Ok(())
    }
}
```

### 6. **TUI Dashboard** (`src/ui/dashboard.rs`)

```rust
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct Dashboard {
    state: ExecutionState,
    output: String,
}

impl Dashboard {
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.size());
        
        // Header
        let header = Paragraph::new(format!("BL1NK - Phase: {:?}", self.state.phase))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);
        
        // Main output
        let output = Paragraph::new(self.output.clone())
            .block(Block::default().borders(Borders::ALL).title("Output"));
        f.render_widget(output, chunks[1]);
        
        // Footer
        let footer = Paragraph::new("Press 'q' to quit | 'p' to pause")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}
```

---

## 🔧 Cargo.toml Dependencies

```toml
[package]
name = "blink-nexus"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI & Async
tokio = { version = "1.35", features = ["full"] }
clap = { version = "4.4", features = ["derive"] }

# TUI
ratatui = "0.26"
crossterm = "0.27"

# Git Operations
git2 = "0.18"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Async Traits
async-trait = "0.1"

# HTTP Client
reqwest = { version = "0.11", features = ["json", "stream"] }

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Validation (like Zod)
validator = "0.16"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Config
dirs = "5.0"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
```

---

## 📊 Data Flow Comparison

| Aspect | TypeScript | Rust |
|--------|-----------|------|
| **Runtime** | Node.js | Tokio (async) |
| **CLI Parser** | Commander.js | Clap |
| **TUI** | @opentui/core | ratatui |
| **State Mgmt** | Zod + JSON | Serde + Structs |
| **Git Ops** | simple-git | git2-rs |
| **Async** | Promise/async-await | async/await + Tokio |
| **Type Safety** | TypeScript | Rust (compile-time) |
| **Performance** | Good | Excellent |
| **Memory** | ~100MB | ~10-20MB |

---

## 🚀 Execution Flow (Rust)

```
main.rs (Clap Parser)
    ↓
[TUI Mode] → ui/dashboard.rs (ratatui)
[Headless] → executor.rs (direct)
    ↓
SequentialExecutor::execute()
    ↓
Loop through phases:
  - execute_brief()
  - execute_decompose()
  - execute_investigate()
  - execute_design()
  - execute_execute()
  - execute_optimize()
  - execute_complete()
    ↓
Provider::execute() (async channel)
    ↓
GitSandbox operations
    ↓
State persistence (serde_json)
```

---

## ✨ Rust-specific Advantages

1. **Memory Safety**: No null pointer exceptions, no data races
2. **Performance**: Near C-level performance, zero-cost abstractions
3. **Concurrency**: Tokio for efficient async operations
4. **Type System**: Compile-time guarantees vs runtime validation
5. **Error Handling**: Result<T> forces explicit error handling
6. **Binary Distribution**: Single executable, no runtime dependency

นี่คือ BL1NK Architecture ที่ปรับให้เหมาะสมกับ Rust! 🦀