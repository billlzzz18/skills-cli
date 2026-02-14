  # BL1NK Bridge: Unified Skill System

  ## Overview

  **BL1NK Bridge** is a comprehensive skill orchestration system that enables:

  1. **CLI Invocation**: Execute BL1NK tasks via command-line with structured JSON responses
  2. **Database Access**: Query execution history, sessions, and artifacts from SQLite
  3. **MCP Bridge**: Real-time bidirectional communication for streaming, permissions, and custom tools
  4. **Artifact Management**: Store and retrieve generated code, documents, and configurations

  ---

  ## 1. CLI Invocation

  ### Command Structure

  ```bash
  blink <command> [options]
  ```

  ### Available Commands

  | Command | Purpose | Example |
  |---------|---------|---------|
  | `execute` | Run a task with specified prompt | `blink execute -p "Build API" -f json` |
  | `session` | Manage execution sessions | `blink session list` |
  | `artifact` | Retrieve generated artifacts | `blink artifact get <id>` |
  | `config` | Manage settings | `blink config set provider gemini` |
  | `resume` | Resume interrupted execution | `blink resume <session-id>` |

  ### Flags

  | Flag | Long | Type | Description |
  |------|------|------|-------------|
  | `-p` | `--prompt` | string | Task prompt/description |
  | `-f` | `--format` | enum | Output format: `json`, `text`, `stream` |
  | `-q` | `--quiet` | bool | Suppress spinner/progress |
  | `-c` | `--cwd` | path | Working directory |
  | `-d` | `--debug` | bool | Enable debug logging |
  | `--session-id` | - | uuid | Resume specific session |
  | `--provider` | - | string | Override default provider |
  | `--model` | - | string | Specify model version |
  | `--timeout` | - | int | Execution timeout (seconds) |

  ### Example Invocations

  #### Non-interactive execution
  ```bash
  blink execute \
    -p "Create a REST API for user management" \
    -f json \
    -q \
    -c ~/projects/myapp
  ```

  #### Streaming output
  ```bash
  blink execute \
    -p "Optimize database queries" \
    -f stream \
    --provider gemini \
    --model gemini-2.0-flash
  ```

  #### Resume session
  ```bash
  blink resume abc123-def456 \
    -f json \
    --timeout 3600
  ```

  ### Response Format (JSON)

  ```json
  {
    "success": true,
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "phase": "COMPLETE",
    "content": "Task completed successfully...",
    "artifacts": [
      {
        "id": "art_001",
        "type": "code",
        "name": "api.rs",
        "size": 2048,
        "created_at": 1704067200
      }
    ],
    "metadata": {
      "duration_ms": 45230,
      "tokens_used": 12450,
      "cost_usd": 0.18,
      "provider": "gemini",
      "model": "gemini-2.0-flash"
    },
    "messages": [
      {
        "id": "msg_001",
        "role": "assistant",
        "parts": [...]
      }
    ]
  }
  ```

  ### Streaming Response (JSON-Lines)

  ```json
  {"type":"phase_start","phase":"BRIEF","timestamp":1704067200}
  {"type":"progress","percentage":15,"message":"Analyzing requirements..."}
  {"type":"stream","content":"Generating PRD..."}
  {"type":"artifact","id":"art_001","name":"requirements.md"}
  {"type":"phase_complete","phase":"BRIEF","duration_ms":5230}
  {"type":"phase_start","phase":"DECOMPOSE","timestamp":1704067205}
  ...
  {"type":"complete","session_id":"550e8400-e29b-41d4-a716-446655440000"}
  ```

  ---

  ## 2. Database Schema

  ### Location

  ```
  ~/.blink/blink.db              # Global database
  .blink/blink.db                # Project-local database
  ```

  ### Core Tables

  #### sessions
  ```sql
  CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    parent_session_id TEXT,
    title TEXT,
    description TEXT,
    status TEXT,  -- 'pending', 'running', 'completed', 'failed', 'paused'
    current_phase TEXT,
    message_count INTEGER,
    artifact_count INTEGER,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    total_tokens INTEGER,
    cost_usd REAL,
    provider TEXT,
    model TEXT,
    duration_ms INTEGER,
    created_at INTEGER,
    updated_at INTEGER,
    completed_at INTEGER,
    error_message TEXT,
    metadata TEXT  -- JSON
  );
  ```

  #### messages
  ```sql
  CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    role TEXT,  -- 'user', 'assistant', 'system'
    parts TEXT, -- JSON array of content parts
    model TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
  );
  ```

  #### artifacts
  ```sql
  CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    message_id TEXT,
    type TEXT,  -- 'code', 'document', 'config', 'binary'
    name TEXT,
    content_hash TEXT,
    size INTEGER,
    mime_type TEXT,
    language TEXT,  -- for code artifacts
    path TEXT,  -- relative path in project
    created_at INTEGER,
    updated_at INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id),
    FOREIGN KEY (message_id) REFERENCES messages(id)
  );
  ```

  #### execution_logs
  ```sql
  CREATE TABLE execution_logs (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    phase TEXT,
    event_type TEXT,  -- 'start', 'progress', 'complete', 'error'
    message TEXT,
    details TEXT,  -- JSON
    timestamp INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
  );
  ```

  #### permissions
  ```sql
  CREATE TABLE permissions (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    resource_type TEXT,  -- 'file', 'git', 'network'
    resource_path TEXT,
    action TEXT,  -- 'read', 'write', 'execute'
    status TEXT,  -- 'pending', 'approved', 'denied'
    requested_at INTEGER,
    resolved_at INTEGER,
    resolved_by TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
  );
  ```

  ### Querying Examples

  #### List recent sessions
  ```sql
  SELECT id, title, status, current_phase, created_at
  FROM sessions
  ORDER BY updated_at DESC
  LIMIT 20;
  ```

  #### Get session with messages
  ```sql
  SELECT s.*, m.id as message_id, m.role, m.parts
  FROM sessions s
  LEFT JOIN messages m ON s.id = m.session_id
  WHERE s.id = ?
  ORDER BY m.created_at ASC;
  ```

  #### Get artifacts for session
  ```sql
  SELECT * FROM artifacts
  WHERE session_id = ?
  ORDER BY created_at DESC;
  ```

  ---

  ## 3. MCP Bridge Protocol

  ### Configuration

  File: `~/.blink/mcp-servers.json`

  ```json
  {
    "mcpServers": {
      "blink-core": {
        "type": "stdio",
        "command": "blink-mcp-bridge",
        "args": ["--stdio"],
        "env": {
          "BLINK_HOME": "~/.blink",
          "LOG_LEVEL": "info"
        }
      }
    }
  }
  ```

  ### Message Protocol

  #### Request
  ```json
  {
    "jsonrpc": "2.0",
    "id": "req_001",
    "method": "execute",
    "params": {
      "prompt": "Build API",
      "provider": "gemini",
      "stream": true
    }
  }
  ```

  #### Response
  ```json
  {
    "jsonrpc": "2.0",
    "id": "req_001",
    "result": {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "running"
    }
  }
  ```

  #### Streaming Notification
  ```json
  {
    "jsonrpc": "2.0",
    "method": "notification/progress",
    "params": {
      "session_id": "550e8400-e29b-41d4-a716-446655440000",
      "phase": "EXECUTE",
      "percentage": 65,
      "message": "Implementing features..."
    }
  }
  ```

  ### Supported Methods

  | Method | Purpose | Params |
  |--------|---------|--------|
  | `execute` | Start execution | `{prompt, provider?, model?, stream?}` |
  | `resume` | Resume session | `{session_id, stream?}` |
  | `cancel` | Cancel execution | `{session_id}` |
  | `get_session` | Fetch session data | `{session_id}` |
  | `list_sessions` | List all sessions | `{limit?, offset?}` |
  | `get_artifact` | Retrieve artifact | `{artifact_id}` |
  | `request_permission` | Ask for approval | `{resource_type, resource_path, action}` |

  ---

  ## 4. Message Content Parts

  Messages contain a `parts` JSON array with different content types:

  ### TextContent
  ```json
  {
    "type": "text",
    "text": "Here's the implementation..."
  }
  ```

  ### ExecutionCall
  ```json
  {
    "type": "execution_call",
    "id": "exec_001",
    "phase": "EXECUTE",
    "command": "cargo build",
    "cwd": "/project"
  }
  ```

  ### ExecutionResult
  ```json
  {
    "type": "execution_result",
    "execution_id": "exec_001",
    "exit_code": 0,
    "stdout": "Compiling...",
    "stderr": "",
    "duration_ms": 3240
  }
  ```

  ### ToolCall
  ```json
  {
    "type": "tool_call",
    "id": "tool_001",
    "name": "create_file",
    "input": {
      "path": "src/main.rs",
      "content": "fn main() { ... }"
    }
  }
  ```

  ### ToolResult
  ```json
  {
    "type": "tool_result",
    "tool_call_id": "tool_001",
    "success": true,
    "result": {
      "path": "src/main.rs",
      "size": 1024
    }
  }
  ```

  ### ArtifactReference
  ```json
  {
    "type": "artifact",
    "id": "art_001",
    "name": "api.rs",
    "type": "code",
    "language": "rust",
    "size": 2048
  }
  ```

  ### FinishSignal
  ```json
  {
    "type": "finish",
    "reason": "end_turn",
    "summary": "Task completed successfully",
    "timestamp": 1704067200
  }
  ```

  ---

  ## 5. Integration Examples

  ### Rust Backend (Axum)

  #### CLI Execution Handler
  ```rust
  use std::process::Command;
  use serde_json::json;

  #[tauri::command]
  async fn execute_cli(
      prompt: String,
      format: String,
  ) -> Result<serde_json::Value, String> {
      let output = Command::new("blink")
          .arg("execute")
          .arg("-p")
          .arg(&prompt)
          .arg("-f")
          .arg(&format)
          .arg("-q")
          .output()
          .map_err(|e| e.to_string())?;

      let json_str = String::from_utf8(output.stdout)
          .map_err(|e| e.to_string())?;

      serde_json::from_str(&json_str)
          .map_err(|e| e.to_string())
  }
  ```

  #### Database Query
  ```rust
  use sqlx::SqlitePool;

  #[tauri::command]
  async fn list_sessions(
      pool: tauri::State<'_, SqlitePool>,
  ) -> Result<Vec<Session>, String> {
      let sessions = sqlx::query_as::<_, Session>(
          "SELECT * FROM sessions ORDER BY updated_at DESC LIMIT 20"
      )
      .fetch_all(pool.get_ref())
      .await
      .map_err(|e| e.to_string())?;

      Ok(sessions)
  }
  ```

  ### Next.js Frontend

  #### API Call
  ```typescript
  async function executeTask(prompt: string) {
    const response = await fetch('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt }),
    });

    return response.json();
  }
  ```

  #### Database Query
  ```typescript
  import Database from '@tauri-apps/plugin-sql';

  async function getSessions() {
    const db = await Database.load('sqlite:~/.blink/blink.db');
    const sessions = await db.select<Session[]>(
      'SELECT * FROM sessions ORDER BY updated_at DESC LIMIT 20'
    );
    return sessions;
  }
  ```

  #### MCP Subscription
  ```typescript
  function subscribeToExecution(sessionId: string) {
    const ws = new WebSocket('ws://localhost:3001/mcp');

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (message.params?.session_id === sessionId) {
        updateProgress(message.params);
      }
    };

    return ws;
  }
  ```

  ---

  ## 6. Common Gotchas

  - **Database Locking**: Use read-only access to avoid conflicts with running BL1NK
  - **Message Parts**: JSON-encoded strings must be parsed in the UI
  - **Session IDs**: UUIDs; use as-is for resumption
  - **Artifact Storage**: Large artifacts stored separately; database contains references
  - **Token Counting**: Approximate; actual usage may vary by provider
  - **Cost Calculation**: USD estimates; verify with provider billing
  - **Timeout Handling**: Default 5 minutes; adjust via `--timeout` flag
  - **Permission Prompts**: Must be resolved before execution continues

  ---

  ## 7. Error Handling

  ### Error Response Format
  ```json
  {
    "success": false,
    "error": {
      "code": "EXECUTION_FAILED",
      "message": "Task execution failed",
      "details": {
        "phase": "EXECUTE",
        "reason": "Compilation error",
        "context": "..."
      }
    },
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  }
  ```

  ### Error Codes
  | Code | Meaning |
  |------|---------|
  | `INVALID_PROMPT` | Prompt validation failed |
  | `PROVIDER_ERROR` | AI provider error |
  | `EXECUTION_FAILED` | Task execution failed |
  | `PERMISSION_DENIED` | Access denied |
  | `TIMEOUT` | Execution timeout |
  | `DATABASE_ERROR` | Database operation failed |
  | `INVALID_SESSION` | Session not found |

  ---

  ## 8. First-Time Setup

  ### Verify BL1NK is installed
  ```bash
  which blink
  blink --version
  ```

  ### Verify database exists
  ```bash
  ls ~/.blink/blink.db
  ```

  ### Test CLI invocation
  ```bash
  blink execute -p "Hello world" -f json -q
  ```

  ### Initialize MCP bridge
  ```bash
  blink mcp init
  ```

  ### Verify database schema
  ```bash
  sqlite3 ~/.blink/blink.db ".schema"
  ```