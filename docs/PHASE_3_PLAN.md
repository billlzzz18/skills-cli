# 🚀 BL1NK Agents Manager - Phase 3: Complete Implementation

## 🎯 ทำความเข้าใจ Gemini Extension Architecture

จากตัวอย่างที่ดู (ComputerUse, Vision, Listen, Packet Buddy):

```
extension/
├── commands/           # Slash commands (TOML)
│   └── feature/
│       └── action.toml
├── servers/            # MCP servers (Python/Rust/Node)
│   └── feature_mcp/
│       └── server.py
└── gemini-extension.json  # Manifest
```

**Key Insight:** 
- Commands = UI (what user types)
- Servers = Backend (what executes)

---

## 📋 BL1NK Agents Manager - Target Architecture

```
bl1nk-agents-manager/
├── agents/                    # ✅ Agent library (40+ MD files)
│   ├── architect.md
│   ├── code-generator.md
│   └── ...
├── commands/                  # ✅ Slash commands
│   ├── system-agent/         # Python-based (existing)
│   │   ├── list.toml
│   │   ├── info.toml
│   │   └── switch.toml
│   ├── delegate-task/        # ⬅️ NEW: MCP-based
│   │   └── delegate.toml
│   └── create-agent/         # ⬅️ NEW: MCP-based
│       └── create.toml
├── servers/                   # ⬅️ NEW: MCP Servers
│   ├── agent-orchestrator/   # Rust MCP (task delegation)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── agents/
│   │       └── routing/
│   └── agent-creator/        # Python MCP (agent generation)
│       ├── server.py
│       └── templates/
├── scripts/                   # ✅ Python utilities
├── docs/
├── gemini-extension.json
└── README.md
```

---

## 🔥 Phase 3: Week-by-Week Plan

### **Week 1: MCP Server - Agent Orchestrator (Rust)**

#### Day 1-2: Setup Project Structure

```bash
# Create server directory
mkdir -p servers/agent-orchestrator
cd servers/agent-orchestrator

# Initialize Rust project
cargo init --name bl1nk-orchestrator-mcp

# Add to .gitignore
echo "target/" >> ../../.gitignore
```

**Cargo.toml:**
```toml
[package]
name = "bl1nk-orchestrator-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP Protocol
pmcp = { version = "1.8", features = ["schema-generation"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# Markdown parsing
pulldown-cmark = "0.11"
serde_yaml = "0.9"
```

#### Day 3-4: Implement MCP Server

**src/main.rs:**
```rust
mod agents;
mod routing;
mod tools;

use pmcp::{ServerBuilder, TypedTool};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("🚀 Starting BL1NK Orchestrator MCP Server");

    // Build MCP server
    let server = ServerBuilder::new()
        .name("bl1nk-orchestrator")
        .version(env!("CARGO_PKG_VERSION"))
        .tool("delegate_task", TypedTool::new(
            "delegate_task",
            |args, extra| Box::pin(tools::delegate_task(args, extra))
        ))
        .tool("agent_status", TypedTool::new(
            "agent_status",
            |args, extra| Box::pin(tools::agent_status(args, extra))
        ))
        .tool("list_agents", TypedTool::new(
            "list_agents",
            |args, extra| Box::pin(tools::list_agents(args, extra))
        ))
        .build()?;

    // Run on stdio
    tracing::info!("🎧 Listening on stdio");
    server.run_stdio().await?;

    Ok(())
}
```

**src/agents/mod.rs:**
```rust
mod loader;
mod executor;

pub use loader::{GeminiAgent, load_agent, load_all_agents};
pub use executor::{execute_agent, AgentResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub system_prompt: String,
    pub tags: Vec<String>,
}
```

**src/agents/loader.rs:**
```rust
use super::GeminiAgent;
use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};

/// Load single agent from .md file
pub fn load_agent(path: &Path) -> Result<GeminiAgent> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    
    // Split frontmatter and body
    let (frontmatter, body) = parse_markdown_frontmatter(&content)?;
    
    Ok(GeminiAgent {
        id: frontmatter.get("name")
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' in frontmatter"))?
            .to_string(),
        name: frontmatter.get("name")
            .unwrap()
            .to_string(),
        description: frontmatter.get("description")
            .unwrap_or(&"".to_string())
            .to_string(),
        category: frontmatter.get("category")
            .unwrap_or(&"general".to_string())
            .to_string(),
        system_prompt: body,
        tags: vec![],
    })
}

/// Load all agents from directory
pub fn load_all_agents() -> Result<Vec<GeminiAgent>> {
    let agents_dir = find_agents_directory()?;
    let mut agents = Vec::new();
    
    for entry in fs::read_dir(&agents_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            match load_agent(&path) {
                Ok(agent) => agents.push(agent),
                Err(e) => {
                    tracing::warn!("Failed to load {:?}: {}", path, e);
                }
            }
        }
    }
    
    tracing::info!("Loaded {} agents", agents.len());
    Ok(agents)
}

/// Find agents directory (../../agents from binary)
fn find_agents_directory() -> Result<PathBuf> {
    // Try relative paths
    let candidates = vec![
        PathBuf::from("../../agents"),
        PathBuf::from("../../../agents"),
        PathBuf::from("agents"),
    ];
    
    for path in candidates {
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }
    
    anyhow::bail!("Could not find agents directory");
}

/// Parse YAML frontmatter from markdown
fn parse_markdown_frontmatter(content: &str) -> Result<(serde_yaml::Value, String)> {
    if !content.starts_with("---") {
        return Ok((serde_yaml::Value::Null, content.to_string()));
    }
    
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Ok((serde_yaml::Value::Null, content.to_string()));
    }
    
    let frontmatter = serde_yaml::from_str(parts[1].trim())?;
    let body = parts[2].trim().to_string();
    
    Ok((frontmatter, body))
}
```

**src/agents/executor.rs:**
```rust
use super::GeminiAgent;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_id: String,
    pub output: String,
    pub success: bool,
}

/// Execute agent with given prompt
pub async fn execute_agent(
    agent: &GeminiAgent,
    prompt: &str,
) -> Result<AgentResult> {
    tracing::info!("Executing agent: {}", agent.id);
    
    // For now, return formatted response
    // In production, this would call Gemini API with system_prompt
    
    let output = format!(
        "Agent '{}' would process:\n{}\n\nWith system prompt:\n{}",
        agent.name,
        prompt,
        &agent.system_prompt[..200.min(agent.system_prompt.len())]
    );
    
    Ok(AgentResult {
        agent_id: agent.id.clone(),
        output,
        success: true,
    })
}
```

**src/routing/mod.rs:**
```rust
use crate::agents::GeminiAgent;

pub struct AgentRouter;

impl AgentRouter {
    /// Select best agent for task
    pub fn select_agent(
        task_type: &str,
        _prompt: &str,
        agents: &[GeminiAgent],
    ) -> Option<&GeminiAgent> {
        // Simple routing by category
        agents.iter()
            .find(|a| a.category.contains(task_type))
            .or_else(|| agents.first())
    }
}
```

**src/tools.rs:**
```rust
use crate::agents::{load_all_agents, execute_agent};
use crate::routing::AgentRouter;
use pmcp::{RequestHandlerExtra, Result as PmcpResult};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DelegateTaskArgs {
    pub task_type: String,
    pub prompt: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DelegateTaskOutput {
    pub task_id: String,
    pub agent_id: String,
    pub result: String,
}

pub async fn delegate_task(
    args: DelegateTaskArgs,
    _extra: RequestHandlerExtra,
) -> PmcpResult<DelegateTaskOutput> {
    let agents = load_all_agents()
        .map_err(|e| pmcp::Error::InternalError(e.to_string()))?;
    
    let selected_agent = if let Some(id) = args.agent_id {
        agents.iter().find(|a| a.id == id)
    } else {
        AgentRouter::select_agent(&args.task_type, &args.prompt, &agents)
    }.ok_or_else(|| pmcp::Error::InvalidRequest("No suitable agent found".into()))?;
    
    let result = execute_agent(selected_agent, &args.prompt)
        .await
        .map_err(|e| pmcp::Error::InternalError(e.to_string()))?;
    
    Ok(DelegateTaskOutput {
        task_id: uuid::Uuid::new_v4().to_string(),
        agent_id: selected_agent.id.clone(),
        result: result.output,
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AgentStatusArgs {
    pub task_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AgentStatusOutput {
    pub available_agents: Vec<String>,
    pub agent_count: usize,
}

pub async fn agent_status(
    _args: AgentStatusArgs,
    _extra: RequestHandlerExtra,
) -> PmcpResult<AgentStatusOutput> {
    let agents = load_all_agents()
        .map_err(|e| pmcp::Error::InternalError(e.to_string()))?;
    
    Ok(AgentStatusOutput {
        available_agents: agents.iter().map(|a| a.id.clone()).collect(),
        agent_count: agents.len(),
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAgentsArgs {}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ListAgentsOutput {
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
}

pub async fn list_agents(
    _args: ListAgentsArgs,
    _extra: RequestHandlerExtra,
) -> PmcpResult<ListAgentsOutput> {
    let agents = load_all_agents()
        .map_err(|e| pmcp::Error::InternalError(e.to_string()))?;
    
    Ok(ListAgentsOutput {
        agents: agents.iter().map(|a| AgentInfo {
            id: a.id.clone(),
            name: a.name.clone(),
            category: a.category.clone(),
            description: a.description.clone(),
        }).collect(),
    })
}
```

#### Day 5: Build & Test

```bash
# Build
cargo build --release

# Test manually
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}' | ./target/release/bl1nk-orchestrator-mcp

# Should see: delegate_task, agent_status, list_agents
```

---

### **Week 2: Gemini Commands Integration**

#### Day 1: Create delegate-task Command

**commands/delegate-task/delegate.toml:**
```toml
[command]
name = "delegate-task"
description = "Delegate a task to a specialized agent"
usage = "/delegate-task <prompt> [--agent=<agent_id>] [--type=<task_type>]"

[command.mcp]
server = "bl1nk-orchestrator"
tool = "delegate_task"

# Arguments mapping
[command.mcp.args]
task_type = { type = "string", default = "general", from = "flag:type" }
prompt = { type = "string", required = true, from = "positional:0" }
agent_id = { type = "string", optional = true, from = "flag:agent" }

[command.config]
auto_start_server = true
server_command = "servers/agent-orchestrator/target/release/bl1nk-orchestrator-mcp"
```

#### Day 2: Create agent-status Command

**commands/agent-status/status.toml:**
```toml
[command]
name = "agent-status"
description = "Show status of agents and tasks"
usage = "/agent-status [--task=<task_id>]"

[command.mcp]
server = "bl1nk-orchestrator"
tool = "agent_status"

[command.mcp.args]
task_id = { type = "string", optional = true, from = "flag:task" }

[command.config]
auto_start_server = true
server_command = "servers/agent-orchestrator/target/release/bl1nk-orchestrator-mcp"
```

#### Day 3: Update gemini-extension.json

```json
{
  "name": "bl1nk-agents-manager",
  "version": "0.2.0",
  "description": "Multi-persona AI Team with MCP orchestration",
  "commands": [
    "commands/system-agent",
    "commands/delegate-task",
    "commands/agent-status"
  ],
  "mcpServers": {
    "bl1nk-orchestrator": {
      "command": "servers/agent-orchestrator/target/release/bl1nk-orchestrator-mcp",
      "transport": "stdio"
    }
  }
}
```

#### Day 4-5: Testing

```bash
# Install extension
gemini extensions install /path/to/bl1nk-agents-manager

# Test 1: List agents (existing)
gemini /system-agent

# Test 2: Delegate task (NEW)
gemini /delegate-task "Design a microservices architecture" --type=architecture

# Test 3: Check status (NEW)
gemini /agent-status
```

---

### **Week 3: Agent Creator (Python MCP)**

#### Day 1-2: Python MCP Server

**servers/agent-creator/server.py:**
```python
#!/usr/bin/env python3
import asyncio
import sys
from pathlib import Path
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Agent Creator Implementation
class AgentCreatorServer:
    def __init__(self):
        self.server = Server("agent-creator")
        self.agents_dir = Path(__file__).parent.parent.parent / "agents"
        
        # Register tools
        self.server.list_tools = self.list_tools
        self.server.call_tool = self.call_tool
    
    async def list_tools(self):
        return [
            Tool(
                name="create_agent",
                description="Create a new agent from requirements",
                inputSchema={
                    "type": "object",
                    "properties": {
                        "requirements": {
                            "type": "string",
                            "description": "What the agent should do"
                        },
                        "template": {
                            "type": "string",
                            "description": "Template to use (optional)"
                        }
                    },
                    "required": ["requirements"]
                }
            )
        ]
    
    async def call_tool(self, name, arguments):
        if name == "create_agent":
            return await self.create_agent(
                arguments["requirements"],
                arguments.get("template")
            )
        raise ValueError(f"Unknown tool: {name}")
    
    async def create_agent(self, requirements, template=None):
        # Parse requirements
        agent_id = self.generate_id(requirements)
        
        # Generate content
        content = f"""---
name: {agent_id}
description: {requirements}
category: generated
---

You are an expert {requirements}.

## Responsibilities

1. [Generated based on requirements]
2. ...

## Process

[Generated workflow]
"""
        
        # Write file
        output_path = self.agents_dir / f"{agent_id}.md"
        output_path.write_text(content)
        
        return [TextContent(
            type="text",
            text=f"✅ Agent created: {agent_id}\nFile: {output_path}"
        )]
    
    def generate_id(self, requirements):
        # Simple ID generation
        words = requirements.lower().split()[:3]
        return "-".join(w for w in words if w.isalnum())

async def main():
    server = AgentCreatorServer()
    async with stdio_server() as (read, write):
        await server.server.run(read, write)

if __name__ == "__main__":
    asyncio.run(main())
```

**servers/agent-creator/requirements.txt:**
```
mcp>=0.9.0
```

#### Day 3: Create Command

**commands/create-agent/create.toml:**
```toml
[command]
name = "create-agent"
description = "Create a new agent using AI"
usage = "/create-agent <requirements>"

[command.mcp]
server = "agent-creator"
tool = "create_agent"

[command.mcp.args]
requirements = { type = "string", required = true, from = "positional:0" }

[command.config]
auto_start_server = true
server_command = "python3"
server_args = ["servers/agent-creator/server.py"]
```

#### Day 4-5: Integration Testing

```bash
# Create new agent
gemini /create-agent "validates JSON schemas"

# Verify it was created
ls agents/validates-json-schemas.md

# Use it immediately
gemini /delegate-task "Validate this JSON" --agent=validates-json-schemas
```

---

### **Week 4: Documentation & Polish**

#### Complete README

```markdown
# 🤖 BL1NK Agents Manager

**Multi-Persona AI Team with MCP Orchestration**

## Features

### 1. Agent Library (50+)
- Software Architect
- Code Generator
- Creative Writer
- Pirate Assistant
- And more...

### 2. Commands

#### Agent Management
- `/system-agent` - List all agents
- `/system-agent:info <name>` - View details
- `/system-agent:switch <name>` - Switch persona

#### Task Delegation
- `/delegate-task <prompt>` - Auto-route to best agent
- `/delegate-task <prompt> --agent=<id>` - Use specific agent
- `/agent-status` - View available agents

#### Agent Creation
- `/create-agent <requirements>` - Generate new agent

## Installation

```bash
gemini extensions install https://github.com/billlzzz18/bl1nk-agents-manager
```

## Usage

```bash
# List agents
gemini /system-agent

# Delegate architecture task
gemini /delegate-task "Design a microservices system" --type=architecture

# Create custom agent
gemini /create-agent "optimizes SQL queries"

# Use created agent
gemini /delegate-task "Optimize this SELECT" --agent=sql-optimizer
```
```

---

## ✅ Final Checklist

### Week 1
- [ ] Create Rust MCP server structure
- [ ] Implement agent loader
- [ ] Implement delegate_task tool
- [ ] Build and test locally

### Week 2
- [ ] Create /delegate-task command
- [ ] Create /agent-status command
- [ ] Update gemini-extension.json
- [ ] Integration testing

### Week 3
- [ ] Create Python agent-creator server
- [ ] Implement create_agent tool
- [ ] Create /create-agent command
- [ ] End-to-end testing

### Week 4
- [ ] Update all documentation
- [ ] Create demo video
- [ ] Write examples
- [ ] Release v1.0.0