  /**
  ACP (Agent Client Protocol) implementation without Phoenix.
  Uses JSON-RPC over stdio for communication.
  */
  use anyhow::{anyhow, Result};
  use dashmap::{DashMap, DashSet};
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Map, Value};
  use std::{
      collections::HashMap,
      process::Stdio,
      sync::{
          atomic::{AtomicBool, AtomicU64, Ordering},
          Arc,
      },
  };
  use tokio::{
      io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
      sync::{broadcast, mpsc, Mutex, Notify, RwLock},
  };
  use tracing::{debug, error, info, trace, warn};
  use uuid::Uuid;

  // ============================================================================
  // JSON-RPC Types
  // ============================================================================

  #[derive(Debug, Clone, Serialize, Deserialize)]
  #[serde(untagged)]
  pub enum JsonRpcMessage {
      Request(JsonRpcRequest),
      Response(JsonRpcResponse),
      Notification(JsonRpcNotification),
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JsonRpcRequest {
      pub jsonrpc: String,
      pub id: Value,
      pub method: String,
      pub params: Option<Value>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JsonRpcResponse {
      pub jsonrpc: String,
      pub id: Value,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub result: Option<Value>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub error: Option<JsonRpcError>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JsonRpcNotification {
      pub jsonrpc: String,
      pub method: String,
      pub params: Option<Value>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct JsonRpcError {
      pub code: i32,
      pub message: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub data: Option<Value>,
  }

  // ============================================================================
  // Custom Tidewave Protocol Types
  // ============================================================================

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TidewaveSpawnOptions {
      pub command: String,
      pub env: HashMap<String, String>,
      pub cwd: String,
      #[serde(default)]
      pub is_wsl: bool,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TidewaveSessionLoadRequest {
      #[serde(rename = "sessionId")]
      pub session_id: String,
      #[serde(rename = "latestId")]
      pub latest_id: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TidewaveSessionLoadResponse {
      pub cancelled: bool,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TidewaveAckNotification {
      #[serde(rename = "sessionId")]
      pub session_id: String,
      #[serde(rename = "latestId")]
      pub latest_id: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TidewaveExitParams {
      pub error: String,
      pub message: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub stdout: Option<String>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub stderr: Option<String>,
  }

  #[derive(Debug, Clone)]
  pub struct AgentExitEvent {
      pub error: String,
      pub message: String,
      pub stdout: Option<String>,
      pub stderr: Option<String>,
  }

  // ============================================================================
  // Regular ACP Message Types
  // ============================================================================

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct NewSessionResponse {
      #[serde(rename = "sessionId")]
      pub session_id: String,
  }

  // ============================================================================
  // Process Starter Abstraction
  // ============================================================================

  pub type ProcessIo = (
      Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
      Box<dyn tokio::io::AsyncBufRead + Unpin + Send>,
      Box<dyn tokio::io::AsyncBufRead + Unpin + Send>,
      Option<ChildProcess>,
  );

  pub type ProcessStarterFn = Arc<
      dyn Fn(
              TidewaveSpawnOptions,
          )
              -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ProcessIo>> + Send>>
          + Send
          + Sync,
  >;

  // ============================================================================
  // Server State Types
  // ============================================================================

  pub type ChannelId = Uuid;
  pub type ProcessKey = String;
  pub type SessionId = String;
  pub type NotificationId = String;

  /// Output sender for JSON-RPC responses
  pub struct ResponseSender {
      pub tx: mpsc::UnboundedSender<JsonRpcMessage>,
  }

  #[derive(Clone)]
  pub struct AcpChannelState {
      pub processes: Arc<DashMap<ProcessKey, Arc<ProcessState>>>,
      pub channel_senders: Arc<DashMap<ChannelId, ResponseSender>>,
      pub sessions: Arc<DashMap<SessionId, Arc<SessionState>>>,
      pub session_to_channel: Arc<DashMap<SessionId, ChannelId>>,
      pub process_starter: ProcessStarterFn,
      pub process_lifecycle_locks: Arc<DashMap<ProcessKey, Arc<Mutex<()>>>>,
  }

  impl AcpChannelState {
      pub fn new() -> Self {
          Self::with_process_starter(real_process_starter())
      }

      pub fn with_process_starter(process_starter: ProcessStarterFn) -> Self {
          Self {
              processes: Arc::new(DashMap::new()),
              channel_senders: Arc::new(DashMap::new()),
              sessions: Arc::new(DashMap::new()),
              session_to_channel: Arc::new(DashMap::new()),
              process_starter,
              process_lifecycle_locks: Arc::new(DashMap::new()),
          }
      }
  }

  pub struct ProcessState {
      pub key: ProcessKey,
      pub spawn_opts: TidewaveSpawnOptions,
      pub child: Arc<RwLock<Option<ChildProcess>>>,
      pub stdin_tx: Arc<RwLock<Option<mpsc::UnboundedSender<JsonRpcMessage>>>>,
      pub exit_tx: Arc<RwLock<Option<mpsc::UnboundedSender<()>>>>,
      pub exit_broadcast: broadcast::Sender<AgentExitEvent>,
      pub next_proxy_id: Arc<AtomicU64>,

      pub client_to_proxy_ids: Arc<DashMap<(ChannelId, Value), Value>>,
      pub proxy_to_client_ids: Arc<DashMap<Value, (ChannelId, Value)>>,
      pub proxy_to_session_ids: Arc<DashMap<Value, (SessionId, Value)>>,

      pub cached_init_response: Arc<RwLock<Option<JsonRpcResponse>>>,
      pub init_sent: AtomicBool,
      pub init_complete: Arc<Notify>,

      pub stdout_buffer: Arc<RwLock<Vec<String>>>,
      pub stderr_buffer: Arc<RwLock<Vec<String>>>,

      pub init_request_id: Arc<RwLock<Option<Value>>>,
      pub new_request_ids: Arc<DashSet<Value>>,
      pub load_request_ids: Arc<DashMap<Value, SessionId>>,
      pub resume_request_ids: Arc<DashMap<Value, SessionId>>,
      pub fork_request_ids: Arc<DashSet<Value>>,

      pub connect_epoch: Arc<AtomicU64>,
      pub supports_resuming: Arc<RwLock<Option<bool>>>,
  }

  pub struct SessionState {
      pub process_key: ProcessKey,
      pub message_buffer: Arc<RwLock<Vec<BufferedMessage>>>,
      pub notification_id_counter: Arc<AtomicU64>,
      pub cancelled: Arc<AtomicBool>,
      pub cancel_counter: Arc<AtomicU64>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct BufferedMessage {
      pub id: NotificationId,
      pub message: JsonRpcMessage,
  }

  impl ProcessState {
      pub fn new(key: ProcessKey, spawn_opts: TidewaveSpawnOptions) -> Self {
          let (exit_broadcast, _) = broadcast::channel::<AgentExitEvent>(1);
          Self {
              key,
              spawn_opts,
              child: Arc::new(RwLock::new(None)),
              stdin_tx: Arc::new(RwLock::new(None)),
              exit_tx: Arc::new(RwLock::new(None)),
              exit_broadcast,
              next_proxy_id: Arc::new(AtomicU64::new(1)),
              client_to_proxy_ids: Arc::new(DashMap::new()),
              proxy_to_client_ids: Arc::new(DashMap::new()),
              proxy_to_session_ids: Arc::new(DashMap::new()),
              cached_init_response: Arc::new(RwLock::new(None)),
              init_sent: AtomicBool::new(false),
              init_complete: Arc::new(Notify::new()),
              stdout_buffer: Arc::new(RwLock::new(Vec::new())),
              stderr_buffer: Arc::new(RwLock::new(Vec::new())),
              init_request_id: Arc::new(RwLock::new(None)),
              new_request_ids: Arc::new(DashSet::<Value>::new()),
              load_request_ids: Arc::new(DashMap::new()),
              resume_request_ids: Arc::new(DashMap::new()),
              fork_request_ids: Arc::new(DashSet::<Value>::new()),
              connect_epoch: Arc::new(AtomicU64::new(0)),
              supports_resuming: Arc::new(RwLock::new(None)),
          }
      }

      pub async fn send_to_process(&self, message: JsonRpcMessage) -> Result<()> {
          if let Some(tx) = self.stdin_tx.read().await.as_ref() {
              tx.send(message)
                  .map_err(|e| anyhow!("Failed to send message to process: {}", e))?;
          } else {
              return Err(anyhow!("Process stdin channel not available"));
          }
          Ok(())
      }

      pub fn generate_proxy_id(&self) -> Value {
          let id = self.next_proxy_id.fetch_add(1, Ordering::SeqCst);
          Value::Number(serde_json::Number::from(id))
      }

      pub fn map_client_id_to_proxy(
          &self,
          channel_id: ChannelId,
          client_id: Value,
          session_id: Option<SessionId>,
      ) -> Value {
          let proxy_id = self.generate_proxy_id();
          self.client_to_proxy_ids
              .insert((channel_id, client_id.clone()), proxy_id.clone());
          self.proxy_to_client_ids
              .insert(proxy_id.clone(), (channel_id, client_id.clone()));

          if let Some(session_id) = session_id {
              self.proxy_to_session_ids
                  .insert(proxy_id.clone(), (session_id, client_id));
          }
          proxy_id
      }

      pub fn resolve_proxy_id_to_client(&self, proxy_id: &Value) -> Option<(ChannelId, Value)> {
          self.proxy_to_client_ids
              .get(proxy_id)
              .map(|entry| entry.value().clone())
      }

      pub fn cleanup_id_mappings(&self, proxy_id: &Value) {
          if let Some((_, (channel_id, client_id))) = self.proxy_to_client_ids.remove(proxy_id) {
              self.client_to_proxy_ids.remove(&(channel_id, client_id));
          }
          self.proxy_to_session_ids.remove(proxy_id);
      }
  }

  impl SessionState {
      pub fn new(process_key: ProcessKey) -> Self {
          Self {
              process_key,
              message_buffer: Arc::new(RwLock::new(Vec::new())),
              notification_id_counter: Arc::new(AtomicU64::new(1)),
              cancelled: Arc::new(AtomicBool::new(false)),
              cancel_counter: Arc::new(AtomicU64::new(0)),
          }
      }

      pub fn generate_notification_id(&self) -> NotificationId {
          let id = self.notification_id_counter.fetch_add(1, Ordering::SeqCst);
          format!("notif_{}", id)
      }

      pub async fn add_to_buffer(
          &self,
          message: JsonRpcMessage,
          id: NotificationId,
      ) -> NotificationId {
          let buffered = BufferedMessage {
              id: id.clone(),
              message,
          };

          let mut buffer = self.message_buffer.write().await;
          buffer.push(buffered);

          id
      }

      pub async fn prune_buffer(&self, latest_id: &str) {
          let mut buffer = self.message_buffer.write().await;

          if let Some(index) = buffer.iter().position(|msg| msg.id == latest_id) {
              buffer.drain(0..=index);
          }
      }

      pub fn get_buffered_messages_after(
          buffer: &[BufferedMessage],
          latest_id: &str,
      ) -> Vec<BufferedMessage> {
          if let Some(index) = buffer.iter().position(|msg| msg.id == latest_id) {
              buffer.iter().skip(index + 1).cloned().collect()
          } else {
              buffer.iter().cloned().collect()
          }
      }
  }

  // ============================================================================
  // Output Helpers
  // ============================================================================

  fn send_response(sender: &ResponseSender, message: &JsonRpcMessage) {
      if let Err(e) = sender.tx.send(message.clone()) {
          error!("Failed to send response: {}", e);
      }
  }

  // ============================================================================
  // Client Message Handlers
  // ============================================================================

  pub async fn handle_client_message(
      state: &AcpChannelState,
      channel_id: ChannelId,
      message: JsonRpcMessage,
      process_key: &ProcessKey,
  ) -> Result<()> {
      match &message {
          JsonRpcMessage::Request(req) => {
              debug!("Handling request: {} with method {}", req.id, req.method);
              handle_client_request(state, channel_id, req, process_key).await
          }
          JsonRpcMessage::Notification(notif) => {
              debug!("Handling notification with method {}", notif.method);
              handle_client_notification(state, channel_id, notif, process_key).await
          }
          JsonRpcMessage::Response(resp) => {
              debug!("Forwarding response for ID {} to process", resp.id);
              forward_response_to_process(state, resp, process_key).await
          }
      }
  }

  async fn handle_client_request(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
      process_key: &ProcessKey,
  ) -> Result<()> {
      match request.method.as_str() {
          "initialize" => handle_initialize_request(state, channel_id, request, process_key).await,
          "_tidewave.ai/session/load" => {
              handle_tidewave_session_load(state, channel_id, request).await
          }
          "session/load" => handle_acp_session_load(state, channel_id, request, process_key).await,
          _ => handle_regular_request(state, channel_id, request, process_key).await,
      }
  }

  async fn handle_initialize_request(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
      process_key: &ProcessKey,
  ) -> Result<()> {
      let process_state = ensure_process(state, process_key)?;

      if let Some(cached_response) = process_state.cached_init_response.read().await.as_ref() {
          let mut response = cached_response.clone();
          response.id = request.id.clone();
          send_to_channel(state, channel_id, JsonRpcMessage::Response(response));
          return Ok(());
      }

      if process_state
          .init_sent
          .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
          .is_ok()
      {
          let session_id = extract_session_id_from_request(request);
          let proxy_id =
              process_state.map_client_id_to_proxy(channel_id, request.id.clone(), session_id);
          let mut proxy_request = request.clone();
          proxy_request.id = proxy_id.clone();

          *process_state.init_request_id.write().await = Some(proxy_id);

          if let Err(e) = process_state
              .send_to_process(JsonRpcMessage::Request(proxy_request))
              .await
          {
              error!("Failed to send initialize request to process: {}", e);
              send_agent_exit(
                  state,
                  channel_id,
                  "communication_error",
                  "Failed to communicate with process",
                  None,
                  None,
              );
          }
      } else {
          process_state.init_complete.notified().await;

          if let Some(cached_response) = process_state.cached_init_response.read().await.as_ref() {
              let mut response = cached_response.clone();
              response.id = request.id.clone();
              send_to_channel(state, channel_id, JsonRpcMessage::Response(response));
          } else {
              send_agent_exit(
                  state,
                  channel_id,
                  "init_error",
                  "Process init failed",
                  None,
                  None,
              );
          }
      }

      Ok(())
  }

  async fn handle_tidewave_session_load(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
  ) -> Result<()> {
      let params: TidewaveSessionLoadRequest =
          serde_json::from_value(request.params.clone().unwrap_or(Value::Null))
              .map_err(|e| anyhow!("Invalid session/load params: {}", e))?;

      let session_state = match state.sessions.get(&params.session_id) {
          Some(session) => {
              let s = session.clone();
              s.cancel_counter.fetch_add(1, Ordering::SeqCst);
              s
          }
          None => {
              send_error_response(
                  state,
                  channel_id,
                  &request.id,
                  JsonRpcError {
                      code: -32002,
                      message: "Session not found".to_string(),
                      data: None,
                  },
              );
              return Ok(());
          }
      };

      if !ensure_session_not_active(state, channel_id, request, &params.session_id) {
          return Ok(());
      }

      let was_cancelled = session_state.cancelled.swap(false, Ordering::SeqCst);

      if let Some(sender) = state.channel_senders.get(&channel_id) {
          {
              let buffer = session_state.message_buffer.read().await;

              let buffered_messages =
                  SessionState::get_buffered_messages_after(&buffer, &params.latest_id);

              for buffered in buffered_messages {
                  send_response(&sender, &buffered.message);
              }

              state
                  .session_to_channel
                  .insert(params.session_id.clone(), channel_id);
          }

          let response_data = TidewaveSessionLoadResponse {
              cancelled: was_cancelled,
          };

          let success_response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: request.id.clone(),
              result: serde_json::to_value(response_data).ok(),
              error: None,
          };

          send_response(&sender, &JsonRpcMessage::Response(success_response));
      }

      Ok(())
  }

  async fn handle_acp_session_load(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
      process_key: &ProcessKey,
  ) -> Result<()> {
      let session_id = extract_session_id_from_request(request);

      if let Some(session_id) = session_id {
          if !ensure_session_not_active(state, channel_id, request, &session_id) {
              return Ok(());
          }

          if state.sessions.contains_key(&session_id) {
              info!(
                  "session/load for existing session {} on channel {}",
                  session_id, channel_id
              );
          } else {
              let session_state = Arc::new(SessionState::new(process_key.clone()));
              state.sessions.insert(session_id.clone(), session_state);

              info!(
                  "Created new session {} for session/load on channel {}",
                  session_id, channel_id
              );
          }

          state
              .session_to_channel
              .insert(session_id.clone(), channel_id);

          info!(
              "Mapped channel {} to session {} for session/load",
              channel_id, session_id
          );
      }

      handle_regular_request(state, channel_id, request, process_key).await
  }

  async fn handle_regular_request(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
      process_key: &ProcessKey,
  ) -> Result<()> {
      let process_state = ensure_process(state, process_key)?;

      let session_id = extract_session_id_from_request(request);
      let proxy_id =
          process_state.map_client_id_to_proxy(channel_id, request.id.clone(), session_id.clone());
      let mut proxy_request = request.clone();
      proxy_request.id = proxy_id.clone();

      match request.method.as_str() {
          "session/new" => {
              process_state.new_request_ids.insert(proxy_id.clone());
          }
          "session/load" => {
              if let Some(session_id) = session_id {
                  process_state
                      .load_request_ids
                      .insert(proxy_id.clone(), session_id);
              }
          }
          "session/resume" => {
              if let Some(session_id) = session_id {
                  process_state
                      .resume_request_ids
                      .insert(proxy_id.clone(), session_id);
              }
          }
          "session/fork" => {
              process_state.fork_request_ids.insert(proxy_id.clone());
          }
          _ => (),
      }

      if let Err(e) = process_state
          .send_to_process(JsonRpcMessage::Request(proxy_request))
          .await
      {
          error!("Failed to send request to process: {}", e);
      }

      Ok(())
  }

  async fn handle_client_notification(
      state: &AcpChannelState,
      channel_id: ChannelId,
      notification: &JsonRpcNotification,
      process_key: &ProcessKey,
  ) -> Result<()> {
      match notification.method.as_str() {
          "_tidewave.ai/ack" => handle_ack_notification(state, channel_id, notification).await,
          _ => forward_notification_to_process(state, notification, process_key).await,
      }
  }

  async fn handle_ack_notification(
      state: &AcpChannelState,
      channel_id: ChannelId,
      notification: &JsonRpcNotification,
  ) -> Result<()> {
      let params: TidewaveAckNotification =
          serde_json::from_value(notification.params.clone().unwrap_or(Value::Null))
              .map_err(|e| anyhow!("Invalid ack params: {}", e))?;

      if let Some(session_state) = state.sessions.get(&params.session_id) {
          if let Some(mapped_channel_id) = state.session_to_channel.get(&params.session_id) {
              if *mapped_channel_id == channel_id {
                  session_state.prune_buffer(&params.latest_id).await;
              } else {
                  warn!(
                      "Channel {} tried to ACK session {} but is not the owner",
                      channel_id, params.session_id
                  );
              }
          }
      } else {
          warn!("ACK for unknown session: {}", params.session_id);
      }

      Ok(())
  }

  async fn forward_notification_to_process(
      state: &AcpChannelState,
      notification: &JsonRpcNotification,
      process_key: &ProcessKey,
  ) -> Result<()> {
      let process_state = ensure_process(state, process_key)?;

      process_state
          .send_to_process(JsonRpcMessage::Notification(notification.clone()))
          .await?;

      Ok(())
  }

  async fn forward_response_to_process(
      state: &AcpChannelState,
      response: &JsonRpcResponse,
      process_key: &ProcessKey,
  ) -> Result<()> {
      let process_state = ensure_process(state, process_key)?;

      process_state
          .send_to_process(JsonRpcMessage::Response(response.clone()))
          .await?;

      Ok(())
  }

  // ============================================================================
  // Process Management
  // ============================================================================

  pub async fn start_acp_process(
      process_state: Arc<ProcessState>,
      state: AcpChannelState,
  ) -> Result<()> {
      let (stdin, stdout, stderr, child) =
          (state.process_starter)(process_state.spawn_opts.clone()).await?;

      *process_state.child.write().await = child;

      let (stdin_sender, mut stdin_receiver) = mpsc::unbounded_channel::<JsonRpcMessage>();
      *process_state.stdin_tx.write().await = Some(stdin_sender);

      let mut stdin_writer = stdin;
      tokio::spawn(async move {
          while let Some(message) = stdin_receiver.recv().await {
              if let Ok(json_str) = serde_json::to_string(&message) {
                  let json_line = format!("{}\n", json_str);
                  if let Err(e) = stdin_writer.write_all(json_line.as_bytes()).await {
                      error!("Failed to write to process stdin: {}", e);
                      break;
                  }
                  if let Err(e) = stdin_writer.flush().await {
                      error!("Failed to flush process stdin: {}", e);
                      break;
                  }
              }
          }
          debug!("Process stdin handler ended");
      });

      let process_state_clone = process_state.clone();
      let state_clone = state.clone();
      tokio::spawn(async move {
          let mut lines = stdout.lines();
          while let Ok(Some(line)) = lines.next_line().await {
              if let Ok(message) = serde_json::from_str::<JsonRpcMessage>(&line) {
                  if let Err(e) =
                      handle_process_message(&process_state_clone, &state_clone, message).await
                  {
                      error!("Failed to handle process message: {}", e);
                  }
              } else {
                  debug!("Received non-JSON line from process: {}", line);
                  if process_state_clone
                      .cached_init_response
                      .read()
                      .await
                      .is_none()
                  {
                      process_state_clone.stdout_buffer.write().await.push(line);
                  }
              }
          }
          debug!("Process stdout handler ended");
      });

      let process_state_stderr = process_state.clone();
      tokio::spawn(async move {
          let mut lines = stderr.lines();
          while let Ok(Some(line)) = lines.next_line().await {
              debug!("Process stderr: {}", line);
              if process_state_stderr
                  .cached_init_response
                  .read()
                  .await
                  .is_none()
              {
                  process_state_stderr.stderr_buffer.write().await.push(line);
              }
          }
          debug!("Process stderr handler ended");
      });

      let (exit_tx, mut exit_rx) = mpsc::unbounded_channel::<()>();
      *process_state.exit_tx.write().await = Some(exit_tx);

      let process_state_exit = process_state.clone();
      let state_exit = state.clone();
      tokio::spawn(async move {
          let exit_reason = {
              let mut child_guard = process_state_exit.child.write().await;
              if let Some(process) = child_guard.as_mut() {
                  tokio::select! {
                      status = process.child.wait() => {
                          match status {
                              Ok(s) => {
                                  if s.success() {
                                      debug!("Process exited successfully");
                                      ("process_exit", format!("ACP process exited with code {}", s.code().unwrap_or(0)))
                                  } else {
                                      debug!("Process exited with status: {}", s);
                                      ("process_exit", format!("ACP process exited with code {}", s.code().unwrap_or(-1)))
                                  }
                              }
                              Err(e) => {
                                  error!("Failed to wait for process: {}", e);
                                  ("process_exit", format!("ACP process failed: {}", e))
                              }
                          }
                      }
                      _ = exit_rx.recv() => {
                          debug!("Exit signal received for process: {}", process_state_exit.key);
                          child_guard.take();
                          ("exit_requested", "ACP process was stopped by exit request".to_string())
                      }
                  }
              } else {
                  return;
              }
          };

          let (error_type, exit_message) = exit_reason;

          let (stdout, stderr) = if process_state_exit
              .cached_init_response
              .read()
              .await
              .is_none()
          {
              let stdout_buf = process_state_exit.stdout_buffer.read().await;
              let stderr_buf = process_state_exit.stderr_buffer.read().await;
              (
                  if stdout_buf.is_empty() {
                      None
                  } else {
                      Some(stdout_buf.join("\n"))
                  },
                  if stderr_buf.is_empty() {
                      None
                  } else {
                      Some(stderr_buf.join("\n"))
                  },
              )
          } else {
              (None, None)
          };

          let lock = state_exit
              .process_lifecycle_locks
              .entry(process_state_exit.key.clone())
              .or_insert_with(|| Arc::new(Mutex::new(())))
              .clone();
          let _guard = lock.lock().await;

          let _ = process_state_exit.exit_broadcast.send(AgentExitEvent {
              error: error_type.to_string(),
              message: exit_message,
              stdout,
              stderr,
          });

          let sessions_to_remove: Vec<SessionId> = state_exit
              .sessions
              .iter()
              .filter(|entry| entry.value().process_key == process_state_exit.key)
              .map(|entry| entry.key().clone())
              .collect();

          for session_id in &sessions_to_remove {
              state_exit.sessions.remove(session_id);
          }

          process_state_exit.init_complete.notify_waiters();

          state_exit.processes.remove(&process_state_exit.key);

          debug!(
              "Process exit handler ended, cleaned up {} sessions",
              sessions_to_remove.len()
          );
      });

      Ok(())
  }

  // ============================================================================
  // Helper Functions
  // ============================================================================

  fn send_to_channel(state: &AcpChannelState, channel_id: ChannelId, message: JsonRpcMessage) {
      if let Some(sender) = state.channel_senders.get(&channel_id) {
          send_response(&sender, &message);
      }
  }

  fn send_error_response(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request_id: &Value,
      error: JsonRpcError,
  ) {
      debug!("Sending JSON-RPC error to client: {}", error.message);

      let response = JsonRpcResponse {
          jsonrpc: "2.0".to_string(),
          id: request_id.clone(),
          result: None,
          error: Some(error),
      };

      send_to_channel(state, channel_id, JsonRpcMessage::Response(response));
  }

  fn send_agent_exit(
      state: &AcpChannelState,
      channel_id: ChannelId,
      error_type: &str,
      message: &str,
      stdout: Option<String>,
      stderr: Option<String>,
  ) {
      let exit_params = TidewaveExitParams {
          error: error_type.to_string(),
          message: message.to_string(),
          stdout,
          stderr,
      };

      if let Some(sender) = state.channel_senders.get(&channel_id) {
          match serde_json::to_value(exit_params) {
              Ok(payload) => {
                  let notification = JsonRpcNotification {
                      jsonrpc: "2.0".to_string(),
                      method: "agent_exit".to_string(),
                      params: Some(payload),
                  };
                  send_response(&sender, &JsonRpcMessage::Notification(notification));
              }
              Err(e) => error!("Failed to serialize agent exit params: {}", e),
          }
      }
  }

  fn ensure_session_not_active(
      state: &AcpChannelState,
      channel_id: ChannelId,
      request: &JsonRpcRequest,
      session_id: &str,
  ) -> bool {
      if state.session_to_channel.contains_key(session_id) {
          send_error_response(
              state,
              channel_id,
              &request.id,
              JsonRpcError {
                  code: -32003,
                  message: "Session already has an active connection".to_string(),
                  data: None,
              },
          );
          return false;
      }
      true
  }

  fn ensure_process(state: &AcpChannelState, process_key: &ProcessKey) -> Result<Arc<ProcessState>> {
      match state.processes.get(process_key) {
          Some(process) => Ok(process.clone()),
          None => Err(anyhow!("Process not found for key: {}", process_key)),
      }
  }

  // ============================================================================
  // Process Message Handlers
  // ============================================================================

  async fn handle_process_message(
      process_state: &Arc<ProcessState>,
      state: &AcpChannelState,
      message: JsonRpcMessage,
  ) -> Result<()> {
      debug!("Got process message {:?}", message);
      match &message {
          JsonRpcMessage::Response(response) => {
              handle_process_response(process_state, state, response).await
          }
          JsonRpcMessage::Request(_) | JsonRpcMessage::Notification(_) => {
              handle_process_notification_or_request(state, message).await
          }
      }
  }

  async fn handle_process_response(
      process_state: &Arc<ProcessState>,
      state: &AcpChannelState,
      response: &JsonRpcResponse,
  ) -> Result<()> {
      if let Some((channel_id, client_id)) = process_state.resolve_proxy_id_to_client(&response.id) {
          let mut client_response = response.clone();
          client_response.id = client_id;

          maybe_handle_init_response(process_state, response, &mut client_response).await;
          maybe_handle_session_load_resume(
              process_state,
              state,
              channel_id,
              response,
              &client_response,
          )
          .await?;
          maybe_handle_session_new_response(
              process_state,
              state,
              response,
              &client_response,
              channel_id,
          )
          .await;

          if let Some(sender) = state.channel_senders.get(&channel_id) {
              send_response(&sender, &JsonRpcMessage::Response(client_response));
          } else {
              handle_disconnected_client_response(process_state, state, response).await;
          }

          process_state.cleanup_id_mappings(&response.id);
      }

      Ok(())
  }

  async fn maybe_handle_init_response(
      process_state: &Arc<ProcessState>,
      response: &JsonRpcResponse,
      client_response: &mut JsonRpcResponse,
  ) {
      let init_request_id = process_state.init_request_id.read().await;
      if init_request_id.as_ref() == Some(&response.id) {
          drop(init_request_id);

          *process_state.supports_resuming.write().await = Some(check_supports_resuming(response));

          *process_state.cached_init_response.write().await = Some(client_response.clone());
          process_state.init_complete.notify_waiters();
          *process_state.stderr_buffer.write().await = Vec::new();
          *process_state.stdout_buffer.write().await = Vec::new();
      }
  }

  async fn maybe_handle_session_load_resume(
      process_state: &Arc<ProcessState>,
      state: &AcpChannelState,
      channel_id: ChannelId,
      response: &JsonRpcResponse,
      client_response: &JsonRpcResponse,
  ) -> Result<()> {
      if let Some((_proxy_id, session_id)) = process_state
          .load_request_ids
          .remove(&response.id)
          .or_else(|| process_state.resume_request_ids.remove(&response.id))
      {
          if client_response.error.is_some() {
              info!("Failed to load session, removing mapping! {}", session_id);
              state.sessions.remove(&session_id);
              state.session_to_channel.remove(&session_id);
              if let Some(sender) = state.channel_senders.get(&channel_id) {
                  send_response(&sender, &JsonRpcMessage::Response(client_response.clone()));
              }
              return Err(anyhow!(
                  "Failed to load session, removing mapping! {}",
                  session_id
              ));
          } else {
              map_session_id_to_channel(state, session_id, channel_id, &process_state.key).await;
          }
      }

      Ok(())
  }

  async fn maybe_handle_session_new_response(
      process_state: &Arc<ProcessState>,
      state: &AcpChannelState,
      response: &JsonRpcResponse,
      client_response: &JsonRpcResponse,
      channel_id: ChannelId,
  ) {
      if process_state
          .new_request_ids
          .remove(&response.id)
          .or_else(|| process_state.fork_request_ids.remove(&response.id))
          .is_some()
      {
          if let Some(result) = &client_response.result {
              if let Ok(session_response) =
                  serde_json::from_value::<NewSessionResponse>(result.clone())
              {
                  map_session_id_to_channel(
                      state,
                      session_response.session_id,
                      channel_id,
                      &process_state.key,
                  )
                  .await;
              }
          }
      }
  }

  async fn map_session_id_to_channel(
      state: &AcpChannelState,
      session_id: SessionId,
      channel_id: ChannelId,
      process_key: &ProcessKey,
  ) {
      if !state.sessions.contains_key(&session_id) {
          let session_state = Arc::new(SessionState::new(process_key.clone()));
          state.sessions.insert(session_id.clone(), session_state);
          state.session_to_channel.insert(session_id, channel_id);
      } else {
          warn!(
              "Unexpectedly got new/load/fork session response for already known session! {}",
              session_id
          );
      }
  }

  async fn handle_disconnected_client_response(
      process_state: &Arc<ProcessState>,
      state: &AcpChannelState,
      response: &JsonRpcResponse,
  ) {
      debug!("Missing original channel for request {}", response.id);
      let session_info = process_state
          .proxy_to_session_ids
          .get(&response.id)
          .map(|entry| entry.value().clone());

      if let Some((session_id, client_id)) = session_info {
          let mut client_response = response.clone();
          client_response.id = client_id.clone();

          if let Some(current_channel_id) = state.session_to_channel.get(&session_id) {
              let current_channel_id = *current_channel_id;
              if let Some(sender) = state.channel_senders.get(&current_channel_id) {
                  send_response(&sender, &JsonRpcMessage::Response(client_response));
              }
          } else {
              if let Some(session_state) = state.sessions.get(&session_id) {
                  let session_state = session_state.clone();
                  let _ = session_state
                      .add_to_buffer(
                          JsonRpcMessage::Response(client_response),
                          client_id.to_string(),
                      )
                      .await;
              }
          }

          process_state.proxy_to_session_ids.remove(&response.id);
      }
  }

  async fn handle_process_notification_or_request(
      state: &AcpChannelState,
      message: JsonRpcMessage,
  ) -> Result<()> {
      let session_id = extract_session_id_from_message(&message);

      if let Some(session_id) = session_id {
          if let Some(session_state) = state.sessions.get(&session_id) {
              let session_state = session_state.clone();

              let mut routed_message = message.clone();
              let buffer_id = if let JsonRpcMessage::Notification(ref mut n) = routed_message {
                  let notif_id = session_state.generate_notification_id();
                  inject_notification_id(n, notif_id.clone());
                  notif_id
              } else {
                  match &routed_message {
                      JsonRpcMessage::Request(req) => req.id.to_string(),
                      JsonRpcMessage::Response(resp) => resp.id.to_string(),
                      _ => unreachable!(),
                  }
              };

              let _buffer_id = session_state
                  .add_to_buffer(routed_message.clone(), buffer_id)
                  .await;

              if let Some(channel_id) = state.session_to_channel.get(&session_id) {
                  if let Some(sender) = state.channel_senders.get(&channel_id) {
                      send_response(&sender, &routed_message);
                  }
              }
          } else {
              warn!("Session not found for sessionId: {}", session_id);
          }
      } else {
          warn!(
              "Message from process missing sessionId, ignoring: {:?}",
              message
          );
      }

      Ok(())
  }

  // ============================================================================
  // Utility Functions
  // ============================================================================

  fn check_supports_resuming(response: &JsonRpcResponse) -> bool {
      let caps = response
          .result
          .as_ref()
          .and_then(|r| r.get("agentCapabilities"));

      let Some(caps) = caps else {
          return false;
      };

      if caps.get("loadSession").and_then(|v| v.as_bool()) == Some(true) {
          return true;
      }

      if let Some(session) = caps.get("session").and_then(|v| v.as_object()) {
          if session.contains_key("fork") || session.contains_key("resume") {
              return true;
          }
      }

      false
  }

  fn inject_notification_id(notification: &mut JsonRpcNotification, notif_id: NotificationId) {
      if let Some(params) = &mut notification.params {
          if let Some(params_obj) = params.as_object_mut() {
              let mut meta_obj = params_obj
                  .get("_meta")
                  .and_then(|v| v.as_object())
                  .cloned()
                  .unwrap_or_default();

              meta_obj.insert(
                  "tidewave.ai/notificationId".to_string(),
                  Value::String(notif_id.clone()),
              );
              params_obj.insert("_meta".to_string(), Value::Object(meta_obj));
          }
      } else {
          let mut params_obj = Map::new();
          let mut meta_obj = Map::new();
          meta_obj.insert(
              "tidewave.ai/notificationId".to_string(),
              Value::String(notif_id.clone()),
          );
          params_obj.insert("_meta".to_string(), Value::Object(meta_obj));
          notification.params = Some(Value::Object(params_obj));
      }
  }

  fn extract_session_id_from_message(message: &JsonRpcMessage) -> Option<String> {
      let params = match message {
          JsonRpcMessage::Request(req) => req.params.as_ref(),
          JsonRpcMessage::Response(resp) => resp.result.as_ref(),
          JsonRpcMessage::Notification(notif) => notif.params.as_ref(),
      };

      if let Some(p) = params {
          if let Some(obj) = p.as_object() {
              if let Some(session_id) = obj.get("sessionId") {
                  return session_id.as_str().map(|s| s.to_string());
              }
          }
      }
      None
  }

  fn extract_session_id_from_request(request: &JsonRpcRequest) -> Option<String> {
      if let Some(params) = &request.params {
          if let Some(obj) = params.as_object() {
              if let Some(session_id) = obj.get("sessionId") {
                  return session_id.as_str().map(|s| s.to_string());
              }
          }
      }
      None
  }

  pub fn real_process_starter() -> ProcessStarterFn {
      Arc::new(|spawn_opts: TidewaveSpawnOptions| {
          Box::pin(async move {
              info!("Starting ACP process: {}", spawn_opts.command);

              let mut cmd = create_shell_command(
                  &spawn_opts.command,
                  spawn_opts.env,
                  &spawn_opts.cwd,
                  spawn_opts.is_wsl,
              );

              cmd.stdin(Stdio::piped())
                  .stdout(Stdio::piped())
                  .stderr(Stdio::piped());

              let mut process =
                  spawn_command(cmd).map_err(|e| anyhow!("Failed to spawn process: {}", e))?;

              let stdin = process
                  .child
                  .stdin
                  .take()
                  .ok_or_else(|| anyhow!("Failed to get stdin"))?;
              let stdout = process
                  .child
                  .stdout
                  .take()
                  .ok_or_else(|| anyhow!("Failed to get stdout"))?;
              let stderr = process
                  .child
                  .stderr
                  .take()
                  .ok_or_else(|| anyhow!("Failed to get stderr"))?;

              Ok::<ProcessIo, anyhow::Error>((
                  Box::new(stdin),
                  Box::new(BufReader::new(stdout)),
                  Box::new(BufReader::new(stderr)),
                  Some(process),
              ))
          })
      })
  }
  
    // ============================================================================
  // TESTS
  // ============================================================================

  #[cfg(test)]
  mod tests {
      use super::*;
      use serde_json::json;

      // ========================================================================
      // Buffer Tests
      // ========================================================================

      #[tokio::test]
      async fn test_generate_notification_id() {
          let session = SessionState::new("test_process".to_string());

          let id1 = session.generate_notification_id();
          let id2 = session.generate_notification_id();
          let id3 = session.generate_notification_id();

          assert_eq!(id1, "notif_1");
          assert_eq!(id2, "notif_2");
          assert_eq!(id3, "notif_3");
      }

      #[tokio::test]
      async fn test_add_to_buffer() {
          let session = SessionState::new("test_process".to_string());

          let msg1 = JsonRpcMessage::Notification(JsonRpcNotification {
              jsonrpc: "2.0".to_string(),
              method: "test".to_string(),
              params: None,
          });

          let id1 = session.add_to_buffer(msg1, "notif_1".to_string()).await;

          let buffer = session.message_buffer.read().await;
          assert_eq!(buffer.len(), 1);
          assert_eq!(buffer[0].id, id1);
      }

      #[tokio::test]
      async fn test_prune_buffer() {
          let session = SessionState::new("test_process".to_string());

          for i in 1..=5 {
              let msg = JsonRpcMessage::Notification(JsonRpcNotification {
                  jsonrpc: "2.0".to_string(),
                  method: format!("test_{}", i),
                  params: None,
              });
              session
                  .add_to_buffer(msg, format!("notif_{}", i))
                  .await;
          }

          session.prune_buffer("notif_3").await;

          let buffer = session.message_buffer.read().await;
          assert_eq!(buffer.len(), 2);
          assert_eq!(buffer[0].id, "notif_4");
          assert_eq!(buffer[1].id, "notif_5");
      }

      #[tokio::test]
      async fn test_get_buffered_messages_after() {
          let mut buffer = vec![];

          for i in 1..=5 {
              buffer.push(BufferedMessage {
                  id: format!("notif_{}", i),
                  message: JsonRpcMessage::Notification(JsonRpcNotification {
                      jsonrpc: "2.0".to_string(),
                      method: format!("test_{}", i),
                      params: None,
                  }),
              });
          }

          let result = SessionState::get_buffered_messages_after(&buffer, "notif_2");

          assert_eq!(result.len(), 3);
          assert_eq!(result[0].id, "notif_3");
          assert_eq!(result[1].id, "notif_4");
          assert_eq!(result[2].id, "notif_5");
      }

      #[tokio::test]
      async fn test_get_buffered_messages_after_not_found() {
          let mut buffer = vec![];

          for i in 1..=3 {
              buffer.push(BufferedMessage {
                  id: format!("notif_{}", i),
                  message: JsonRpcMessage::Notification(JsonRpcNotification {
                      jsonrpc: "2.0".to_string(),
                      method: format!("test_{}", i),
                      params: None,
                  }),
              });
          }

          let result = SessionState::get_buffered_messages_after(&buffer, "notif_999");

          assert_eq!(result.len(), 3);
      }

      #[tokio::test]
      async fn test_buffer_workflow() {
          let session = SessionState::new("test_process".to_string());

          // Add 5 messages
          for i in 1..=5 {
              let msg = JsonRpcMessage::Notification(JsonRpcNotification {
                  jsonrpc: "2.0".to_string(),
                  method: format!("test_{}", i),
                  params: None,
              });
              session
                  .add_to_buffer(msg, format!("notif_{}", i))
                  .await;
          }

          let buffer = session.message_buffer.read().await;
          assert_eq!(buffer.len(), 5);
          drop(buffer);

          // Prune up to notif_3
          session.prune_buffer("notif_3").await;

          let buffer = session.message_buffer.read().await;
          assert_eq!(buffer.len(), 2);
          assert_eq!(buffer[0].id, "notif_4");
          assert_eq!(buffer[1].id, "notif_5");
      }

      // ========================================================================
      // ID Mapping Tests
      // ========================================================================

      #[test]
      fn test_generate_proxy_id() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let id1 = process.generate_proxy_id();
          let id2 = process.generate_proxy_id();
          let id3 = process.generate_proxy_id();

          assert_eq!(id1, Value::Number(serde_json::Number::from(1)));
          assert_eq!(id2, Value::Number(serde_json::Number::from(2)));
          assert_eq!(id3, Value::Number(serde_json::Number::from(3)));
      }

      #[test]
      fn test_map_client_id_to_proxy() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let channel_id = Uuid::new_v4();
          let client_id = Value::Number(serde_json::Number::from(100));

          let proxy_id = process.map_client_id_to_proxy(channel_id, client_id.clone(), None);

          assert_eq!(proxy_id, Value::Number(serde_json::Number::from(1)));

          // Verify mapping is stored
          let resolved = process.resolve_proxy_id_to_client(&proxy_id);
          assert!(resolved.is_some());
          let (resolved_channel, resolved_client) = resolved.unwrap();
          assert_eq!(resolved_channel, channel_id);
          assert_eq!(resolved_client, client_id);
      }

      #[test]
      fn test_map_client_id_to_proxy_with_session() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let channel_id = Uuid::new_v4();
          let client_id = Value::Number(serde_json::Number::from(100));
          let session_id = "session_123".to_string();

          let proxy_id = process.map_client_id_to_proxy(
              channel_id,
              client_id.clone(),
              Some(session_id.clone()),
          );

          // Verify session mapping
          let session_mapping = process.proxy_to_session_ids.get(&proxy_id);
          assert!(session_mapping.is_some());
          let (resolved_session, resolved_client) = session_mapping.unwrap().value().clone();
          assert_eq!(resolved_session, session_id);
          assert_eq!(resolved_client, client_id);
      }

      #[test]
      fn test_resolve_proxy_id_to_client() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let channel_id = Uuid::new_v4();
          let client_id = Value::Number(serde_json::Number::from(100));

          let proxy_id = process.map_client_id_to_proxy(channel_id, client_id.clone(), None);

          let resolved = process.resolve_proxy_id_to_client(&proxy_id);
          assert!(resolved.is_some());

          let (resolved_channel, resolved_client) = resolved.unwrap();
          assert_eq!(resolved_channel, channel_id);
          assert_eq!(resolved_client, client_id);
      }

      #[test]
      fn test_resolve_proxy_id_to_client_not_found() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let fake_proxy_id = Value::Number(serde_json::Number::from(999));
          let resolved = process.resolve_proxy_id_to_client(&fake_proxy_id);

          assert!(resolved.is_none());
      }

      #[test]
      fn test_cleanup_id_mappings() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let channel_id = Uuid::new_v4();
          let client_id = Value::Number(serde_json::Number::from(100));

          let proxy_id = process.map_client_id_to_proxy(channel_id, client_id.clone(), None);

          // Verify mapping exists
          assert!(process.resolve_proxy_id_to_client(&proxy_id).is_some());

          // Cleanup
          process.cleanup_id_mappings(&proxy_id);

          // Verify mapping is removed
          assert!(process.resolve_proxy_id_to_client(&proxy_id).is_none());
      }

      #[test]
      fn test_multiple_clients_same_process() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          let channel1 = Uuid::new_v4();
          let channel2 = Uuid::new_v4();

          let client_id1 = Value::Number(serde_json::Number::from(100));
          let client_id2 = Value::Number(serde_json::Number::from(200));

          let proxy_id1 = process.map_client_id_to_proxy(channel1, client_id1.clone(), None);
          let proxy_id2 = process.map_client_id_to_proxy(channel2, client_id2.clone(), None);

          assert_ne!(proxy_id1, proxy_id2);

          let resolved1 = process.resolve_proxy_id_to_client(&proxy_id1).unwrap();
          let resolved2 = process.resolve_proxy_id_to_client(&proxy_id2).unwrap();

          assert_eq!(resolved1.0, channel1);
          assert_eq!(resolved1.1, client_id1);

          assert_eq!(resolved2.0, channel2);
          assert_eq!(resolved2.1, client_id2);
      }

      // ========================================================================
      // Message Extraction Tests
      // ========================================================================

      #[test]
      fn test_extract_session_id_from_request() {
          let request = JsonRpcRequest {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              method: "test".to_string(),
              params: Some(json!({
                  "sessionId": "session_123"
              })),
          };

          let session_id = extract_session_id_from_request(&request);
          assert_eq!(session_id, Some("session_123".to_string()));
      }

      #[test]
      fn test_extract_session_id_from_request_missing() {
          let request = JsonRpcRequest {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              method: "test".to_string(),
              params: Some(json!({})),
          };

          let session_id = extract_session_id_from_request(&request);
          assert_eq!(session_id, None);
      }

      #[test]
      fn test_extract_session_id_from_request_no_params() {
          let request = JsonRpcRequest {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              method: "test".to_string(),
              params: None,
          };

          let session_id = extract_session_id_from_request(&request);
          assert_eq!(session_id, None);
      }

      #[test]
      fn test_extract_session_id_from_message_notification() {
          let message = JsonRpcMessage::Notification(JsonRpcNotification {
              jsonrpc: "2.0".to_string(),
              method: "test".to_string(),
              params: Some(json!({
                  "sessionId": "session_456"
              })),
          });

          let session_id = extract_session_id_from_message(&message);
          assert_eq!(session_id, Some("session_456".to_string()));
      }

      #[test]
      fn test_extract_session_id_from_message_response() {
          let message = JsonRpcMessage::Response(JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({
                  "sessionId": "session_789"
              })),
              error: None,
          });

          let session_id = extract_session_id_from_message(&message);
          assert_eq!(session_id, Some("session_789".to_string()));
      }

      #[test]
      fn test_inject_notification_id() {
          let mut notification = JsonRpcNotification {
              jsonrpc: "2.0".to_string(),
              method: "test".to_string(),
              params: Some(json!({})),
          };

          inject_notification_id(&mut notification, "notif_123".to_string());

          let params = notification.params.unwrap();
          let meta = params.get("_meta").unwrap();
          let notif_id = meta.get("tidewave.ai/notificationId").unwrap();

          assert_eq!(notif_id.as_str().unwrap(), "notif_123");
      }

      #[test]
      fn test_inject_notification_id_no_params() {
          let mut notification = JsonRpcNotification {
              jsonrpc: "2.0".to_string(),
              method: "test".to_string(),
              params: None,
          };

          inject_notification_id(&mut notification, "notif_456".to_string());

          let params = notification.params.unwrap();
          let meta = params.get("_meta").unwrap();
          let notif_id = meta.get("tidewave.ai/notificationId").unwrap();

          assert_eq!(notif_id.as_str().unwrap(), "notif_456");
      }

      #[test]
      fn test_inject_notification_id_preserves_existing_params() {
          let mut notification = JsonRpcNotification {
              jsonrpc: "2.0".to_string(),
              method: "test".to_string(),
              params: Some(json!({
                  "foo": "bar",
                  "baz": 123
              })),
          };

          inject_notification_id(&mut notification, "notif_789".to_string());

          let params = notification.params.unwrap();
          assert_eq!(params.get("foo").unwrap().as_str().unwrap(), "bar");
          assert_eq!(params.get("baz").unwrap().as_i64().unwrap(), 123);

          let meta = params.get("_meta").unwrap();
          let notif_id = meta.get("tidewave.ai/notificationId").unwrap();
          assert_eq!(notif_id.as_str().unwrap(), "notif_789");
      }

      // ========================================================================
      // Capabilities Tests
      // ========================================================================

      #[test]
      fn test_check_supports_resuming_with_load_session() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({
                  "agentCapabilities": {
                      "loadSession": true
                  }
              })),
              error: None,
          };

          assert!(check_supports_resuming(&response));
      }

      #[test]
      fn test_check_supports_resuming_with_session_fork() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({
                  "agentCapabilities": {
                      "session": {
                          "fork": true
                      }
                  }
              })),
              error: None,
          };

          assert!(check_supports_resuming(&response));
      }

      #[test]
      fn test_check_supports_resuming_with_session_resume() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({
                  "agentCapabilities": {
                      "session": {
                          "resume": true
                      }
                  }
              })),
              error: None,
          };

          assert!(check_supports_resuming(&response));
      }

      #[test]
      fn test_check_supports_resuming_no_capabilities() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({})),
              error: None,
          };

          assert!(!check_supports_resuming(&response));
      }

      #[test]
      fn test_check_supports_resuming_false() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: Some(json!({
                  "agentCapabilities": {
                      "loadSession": false,
                      "session": {
                          "fork": false,
                          "resume": false
                      }
                  }
              })),
              error: None,
          };

          assert!(!check_supports_resuming(&response));
      }

      #[test]
      fn test_check_supports_resuming_error_response() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: None,
              error: Some(JsonRpcError {
                  code: -32603,
                  message: "Internal error".to_string(),
                  data: None,
              }),
          };

          assert!(!check_supports_resuming(&response));
      }

      // ========================================================================
      // Integration Tests
      // ========================================================================

      #[test]
      fn test_process_state_creation() {
          let process = ProcessState::new(
              "test_process".to_string(),
              TidewaveSpawnOptions {
                  command: "test".to_string(),
                  env: HashMap::new(),
                  cwd: ".".to_string(),
                  is_wsl: false,
              },
          );

          assert_eq!(process.key, "test_process");
          assert!(!process.init_sent.load(Ordering::SeqCst));
          assert!(process.stdout_buffer.blocking_read().is_empty());
          assert!(process.stderr_buffer.blocking_read().is_empty());
      }

      #[test]
      fn test_session_state_creation() {
          let session = SessionState::new("test_process".to_string());

          assert_eq!(session.process_key, "test_process");
          assert!(!session.cancelled.load(Ordering::SeqCst));
      }

      #[tokio::test]
      async fn test_acp_channel_state_creation() {
          let state = AcpChannelState::new();

          assert!(state.processes.is_empty());
          assert!(state.sessions.is_empty());
          assert!(state.channel_senders.is_empty());
      }

      #[test]
      fn test_tidewave_spawn_options_serialization() {
          let opts = TidewaveSpawnOptions {
              command: "test command".to_string(),
              env: vec![("KEY".to_string(), "value".to_string())]
                  .into_iter()
                  .collect(),
              cwd: "/tmp".to_string(),
              is_wsl: true,
          };

          let json = serde_json::to_value(&opts).unwrap();
          assert_eq!(json.get("command").unwrap().as_str().unwrap(), "test command");
          assert_eq!(json.get("is_wsl").unwrap().as_bool().unwrap(), true);
      }

      #[test]
      fn test_json_rpc_request_serialization() {
          let request = JsonRpcRequest {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              method: "initialize".to_string(),
              params: Some(json!({"test": "value"})),
          };

          let json_str = serde_json::to_string(&request).unwrap();
          let parsed: JsonRpcRequest = serde_json::from_str(&json_str).unwrap();

          assert_eq!(parsed.method, "initialize");
          assert_eq!(parsed.id, Value::Number(serde_json::Number::from(1)));
      }

      #[test]
      fn test_json_rpc_response_with_error() {
          let response = JsonRpcResponse {
              jsonrpc: "2.0".to_string(),
              id: Value::Number(serde_json::Number::from(1)),
              result: None,
              error: Some(JsonRpcError {
                  code: -32603,
                  message: "Internal error".to_string(),
                  data: None,
              }),
          };

          let json_str = serde_json::to_string(&response).unwrap();
          assert!(json_str.contains("error"));
          assert!(!json_str.contains("result"));
      }
  }