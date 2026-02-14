  // src/config/manager.rs - Configuration Management
  
  use super::*;
  use std::path::{Path, PathBuf};
  use tokio::fs;
  use crate::Result;
  
  /// Configuration manager for BL1NK
  #[derive(Clone)]
  pub struct ConfigManager {
      config_dir: PathBuf,
      settings: Settings,
  }
  
  impl ConfigManager {
      /// Load configuration from disk or create default
      pub async fn load() -> Result<Self> {
          let config_dir = Self::config_directory()?;
          fs::create_dir_all(&config_dir).await?;
  
          let settings_path = config_dir.join("settings.json");
          let settings = if settings_path.exists() {
              let content = fs::read_to_string(&settings_path).await?;
              serde_json::from_str(&content)?
          } else {
              Settings::default()
          };
  
          Ok(Self {
              config_dir,
              settings,
          })
      }
  
      /// Save settings to disk
      pub async fn save_settings(&self) -> Result<()> {
          let settings_path = self.config_dir.join("settings.json");
          let content = serde_json::to_string_pretty(&self.settings)?;
          fs::write(&settings_path, content).await?;
          Ok(())
      }
  
      /// Get sessions directory
      pub fn sessions_dir(&self) -> PathBuf {
          self.config_dir.join("sessions")
      }
  
      /// List all sessions
      pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
          let sessions_dir = self.sessions_dir();
          let mut sessions = Vec::new();
  
          if !sessions_dir.exists() {
              return Ok(sessions);
          }
  
          let mut entries = fs::read_dir(&sessions_dir).await?;
          while let Some(entry) = entries.next_entry().await? {
              let path = entry.path();
              if path.is_dir() {
                  if let Ok(metadata) = self.load_session_metadata(&path).await {
                      sessions.push(metadata);
                  }
              }
          }
  
          // Sort by created_at descending
          sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
  
          Ok(sessions)
      }
  
      /// Get configuration directory
      fn config_directory() -> Result<PathBuf> {
          let home = std::env::var("HOME")
              .or_else(|_| std::env::var("USERPROFILE"))?;
          Ok(PathBuf::from(home).join(".blink"))
      }
  
      /// Get reference to settings
      pub fn settings(&self) -> &Settings {
          &self.settings
      }
  
      /// Get mutable reference to settings
      pub fn settings_mut(&mut self) -> &mut Settings {
          &mut self.settings
      }
  
      /// Get configuration directory path
      pub fn config_dir(&self) -> &PathBuf {
          &self.config_dir
      }
  
      // Private helper
      async fn load_session_metadata(&self, path: &Path) -> Result<SessionMetadata> {
          let state_file = path.join("state.json");
          let content = fs::read_to_string(state_file).await?;
          let state: SessionState = serde_json::from_str(&content)?;
          Ok(state.metadata)
      }
  }
  
  // ============================================
  // src/config/mod.rs
  // ============================================
  
  pub mod manager;
  pub mod types;
  
  pub use manager::ConfigManager;
  pub use types::*;