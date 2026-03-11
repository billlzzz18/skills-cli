  use anyhow::{anyhow, Result};
  use regex::Regex;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use serde_yaml::Value as YamlValue;
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};
  use tokio::fs;

  // ============================================================================
  // Models
  // ============================================================================

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Frontmatter {
      pub name: String,
      pub description: Option<String>,
      #[serde(rename = "allowed-tools")]
      pub allowed_tools: Option<serde_json::Value>,
      pub subagents: Option<Vec<Subagent>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Subagent {
      pub name: String,
      pub description: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Command {
      pub name: String,
      pub content: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Marketplace {
      pub metadata: Option<MarketplaceMetadata>,
      pub plugins: Option<Vec<Plugin>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MarketplaceMetadata {
      pub version: Option<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Plugin {
      pub description: Option<String>,
      #[serde(rename = "mcpServers")]
      pub mcp_servers: Option<HashMap<String, MCPServer>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MCPServer {
      pub command: Option<String>,
      pub args: Option<Vec<String>>,
      pub env: Option<HashMap<String, String>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct GeminiManifest {
      pub name: String,
      pub version: String,
      pub description: String,
      #[serde(rename = "contextFileName")]
      pub context_file_name: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      #[serde(rename = "mcpServers")]
      pub mcp_servers: Option<HashMap<String, MCPServer>>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub settings: Option<Vec<Setting>>,
      #[serde(skip_serializing_if = "Option::is_none")]
      #[serde(rename = "excludeTools")]
      pub exclude_tools: Option<Vec<String>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Setting {
      pub name: String,
      pub description: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub default: Option<String>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub secret: Option<bool>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub required: Option<bool>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ConversionResult {
      pub success: bool,
      pub files: Vec<String>,
      pub warnings: Vec<String>,
      pub errors: Vec<String>,
      pub metadata: Option<Metadata>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Metadata {
      pub source: SourceMetadata,
      pub generated: Vec<String>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SourceMetadata {
      pub frontmatter: Option<Frontmatter>,
      pub content: Option<String>,
      pub subagents: Option<Vec<Subagent>>,
      pub commands: Option<Vec<Command>>,
      pub marketplace: Option<Marketplace>,
  }

  // ============================================================================
  // Main Converter
  // ============================================================================

  pub struct ClaudeToGeminiConverter {
      source_path: PathBuf,
      output_path: PathBuf,
      metadata: Metadata,
  }

  impl ClaudeToGeminiConverter {
      pub fn new(source_path: impl AsRef<Path>, output_path: Option<impl AsRef<Path>>) -> Self {
          let source = source_path.as_ref().to_path_buf();
          let output = output_path
              .map(|p| p.as_ref().to_path_buf())
              .unwrap_or_else(|| source.clone());

          Self {
              source_path: source,
              output_path: output,
              metadata: Metadata {
                  source: SourceMetadata {
                      frontmatter: None,
                      content: None,
                      subagents: None,
                      commands: None,
                      marketplace: None,
                  },
                  generated: Vec::new(),
              },
          }
      }

      pub async fn convert(&mut self) -> Result<ConversionResult> {
          let mut result = ConversionResult {
              success: false,
              files: Vec::new(),
              warnings: Vec::new(),
              errors: Vec::new(),
              metadata: None,
          };

          // Step 1: Ensure output directory exists
          fs::create_dir_all(&self.output_path).await?;

          // Step 2: Extract metadata from Claude skill
          if let Err(e) = self.extract_claude_metadata().await {
              result.errors.push(format!("Failed to extract metadata: {}", e));
              return Ok(result);
          }

          // Step 3: Generate gemini-extension.json
          match self.generate_gemini_manifest().await {
              Ok(path) => result.files.push(path),
              Err(e) => result.errors.push(format!("Failed to generate manifest: {}", e)),
          }

          // Step 4: Generate GEMINI.md from SKILL.md
          match self.generate_gemini_context().await {
              Ok(path) => result.files.push(path),
              Err(e) => result.errors.push(format!("Failed to generate context: {}", e)),
          }

          // Step 5: Generate Custom Commands
          match self.generate_commands().await {
              Ok(paths) => result.files.extend(paths),
              Err(e) => result.errors.push(format!("Failed to generate commands: {}", e)),
          }

          // Step 6: Ensure shared directory structure
          if let Err(e) = self.ensure_shared_structure().await {
              result.warnings.push(format!("Failed to create shared structure: {}", e));
          }

          // Step 7: Inject documentation
          if let Err(e) = self.inject_docs().await {
              result.warnings.push(format!("Failed to inject docs: {}", e));
          }

          result.success = result.errors.is_empty();
          result.metadata = Some(self.metadata.clone());

          Ok(result)
      }

      // ========================================================================
      // Extraction Methods
      // ========================================================================

      async fn extract_claude_metadata(&mut self) -> Result<()> {
          // Extract from SKILL.md
          let skill_path = self.source_path.join("SKILL.md");
          let content = fs::read_to_string(&skill_path).await?;

          // Extract YAML frontmatter
          let frontmatter_regex = Regex::new(r"^---\n([\s\S]+?)\n---")?;
          let frontmatter_match = frontmatter_regex
              .captures(&content)
              .ok_or_else(|| anyhow!("SKILL.md missing YAML frontmatter"))?;

          let frontmatter_str = frontmatter_match.get(1).unwrap().as_str();
          let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)?;
          self.metadata.source.frontmatter = Some(frontmatter);

          // Extract content without frontmatter
          let content_without_frontmatter = frontmatter_regex.replace(&content, "").to_string();
          self.metadata.source.content = Some(content_without_frontmatter);

          // Extract subagents if present
          if let Some(ref frontmatter) = self.metadata.source.frontmatter {
              if let Some(ref subagents) = frontmatter.subagents {
                  self.metadata.source.subagents = Some(subagents.clone());
              }
          }

          // Extract Claude slash commands if present
          let commands_dir = self.source_path.join(".claude").join("commands");
          let mut commands = Vec::new();

          if let Ok(mut entries) = fs::read_dir(&commands_dir).await {
              while let Some(entry) = entries.next_entry().await? {
                  let path = entry.path();
                  if path.extension().map_or(false, |ext| ext == "md") {
                      let cmd_content = fs::read_to_string(&path).await?;
                      let name = path
                          .file_stem()
                          .and_then(|s| s.to_str())
                          .unwrap_or("unknown")
                          .to_string();
                      commands.push(Command {
                          name,
                          content: cmd_content,
                      });
                  }
              }
          }

          if !commands.is_empty() {
              self.metadata.source.commands = Some(commands);
          }

          // Extract from marketplace.json if it exists
          let marketplace_path = self
              .source_path
              .join(".claude-plugin")
              .join("marketplace.json");
          if let Ok(marketplace_content) = fs::read_to_string(&marketplace_path).await {
              if let Ok(marketplace) = serde_json::from_str(&marketplace_content) {
                  self.metadata.source.marketplace = Some(marketplace);
              }
          }

          Ok(())
      }

      // ========================================================================
      // Generation Methods
      // ========================================================================

      async fn generate_gemini_manifest(&mut self) -> Result<String> {
          let frontmatter = self
              .metadata
              .source
              .frontmatter
              .as_ref()
              .ok_or_else(|| anyhow!("No frontmatter found"))?;

          let marketplace = self.metadata.source.marketplace.as_ref();

          let version = marketplace
              .and_then(|m| m.metadata.as_ref())
              .and_then(|m| m.version.as_ref())
              .map(|v| v.clone())
              .unwrap_or_else(|| "1.0.0".to_string());

          let description = frontmatter
              .description
              .clone()
              .or_else(|| {
                  marketplace
                      .and_then(|m| m.plugins.as_ref())
                      .and_then(|p| p.first())
                      .and_then(|p| p.description.clone())
              })
              .unwrap_or_default();

          let mut manifest = GeminiManifest {
              name: frontmatter.name.clone(),
              version,
              description,
              context_file_name: "GEMINI.md".to_string(),
              mcp_servers: None,
              settings: None,
              exclude_tools: None,
          };

          // Transform MCP servers configuration
          if let Some(marketplace) = marketplace {
              if let Some(plugins) = &marketplace.plugins {
                  if let Some(plugin) = plugins.first() {
                      if let Some(mcp_servers) = &plugin.mcp_servers {
                          manifest.mcp_servers = Some(self.transform_mcp_servers(mcp_servers));
                      }
                  }
              }
          }

          // Convert allowed-tools to excludeTools
          if let Some(allowed_tools) = &frontmatter.allowed_tools {
              manifest.exclude_tools = Some(self.convert_allowed_tools_to_exclude(allowed_tools));
          }

          // Generate settings from MCP server environment variables
          if let Some(mcp_servers) = &manifest.mcp_servers {
              let settings = self.infer_settings_from_mcp_config(mcp_servers);
              if !settings.is_empty() {
                  manifest.settings = Some(settings);
              }
          }

          // Write to file
          let output_path = self.output_path.join("gemini-extension.json");
          let json_content = serde_json::to_string_pretty(&manifest)?;
          fs::write(&output_path, json_content).await?;

          Ok(output_path.to_string_lossy().to_string())
      }

      fn transform_mcp_servers(
          &self,
          mcp_servers: &HashMap<String, MCPServer>,
      ) -> HashMap<String, MCPServer> {
          let mut transformed = HashMap::new();

          for (server_name, config) in mcp_servers {
              let mut new_config = config.clone();

              // Transform args to use ${extensionPath}
              if let Some(args) = &config.args {
                  new_config.args = Some(
                      args.iter()
                          .map(|arg| {
                              if arg.chars().next().map_or(false, |c| c.is_alphabetic())
                                  && !arg.starts_with("${")
                              {
                                  format!("${{extensionPath}}/{}", arg)
                              } else {
                                  arg.clone()
                              }
                          })
                          .collect(),
                  );
              }

              // Transform env variables to use settings
              if let Some(env) = &config.env {
                  let mut new_env = HashMap::new();
                  for (key, value) in env {
                      let env_var_regex = Regex::new(r"\$\{(.+?)\}").unwrap();
                      if env_var_regex.is_match(value) {
                          new_env.insert(key.clone(), value.clone());
                      } else {
                          new_env.insert(key.clone(), value.clone());
                      }
                  }
                  new_config.env = Some(new_env);
              }

              transformed.insert(server_name.clone(), new_config);
          }

          transformed
      }

      fn convert_allowed_tools_to_exclude(&mut self, allowed_tools: &Value) -> Vec<String> {
          let all_tools = vec![
              "Read", "Write", "Edit", "Glob", "Grep", "Bash", "Task", "WebFetch", "WebSearch",
              "TodoWrite", "AskUserQuestion", "SlashCommand", "Skill", "NotebookEdit",
              "BashOutput", "KillShell",
          ];

          let allowed: Vec<String> = match allowed_tools {
              Value::Array(arr) => arr
                  .iter()
                  .filter_map(|v| v.as_str().map(|s| s.to_string()))
                  .collect(),
              Value::String(s) => s.split(',').map(|t| t.trim().to_string()).collect(),
              _ => Vec::new(),
          };

          let excluded: Vec<String> = all_tools
              .iter()
              .filter(|tool| !allowed.contains(&tool.to_string()))
              .map(|s| s.to_string())
              .collect();

          if excluded.len() > allowed.len() {
              excluded
          } else {
              self.metadata
                  .source
                  .frontmatter
                  .as_mut()
                  .map(|_| {
                      // Add warning
                  });
              Vec::new()
          }
      }

      fn infer_settings_from_mcp_config(
          &self,
          mcp_servers: &HashMap<String, MCPServer>,
      ) -> Vec<Setting> {
          let mut settings = Vec::new();
          let mut seen_vars = std::collections::HashSet::new();
          let env_var_regex = Regex::new(r"\$\{(.+?)\}").unwrap();

          for (_, config) in mcp_servers {
              if let Some(env) = &config.env {
                  for (_, value) in env {
                      if let Some(caps) = env_var_regex.captures(value) {
                          let var_name = caps.get(1).unwrap().as_str();

                          if seen_vars.contains(var_name) {
                              continue;
                          }
                          seen_vars.insert(var_name.to_string());

                          let mut setting = Setting {
                              name: var_name.to_string(),
                              description: self.infer_description(var_name),
                              default: None,
                              secret: None,
                              required: None,
                          };

                          // Detect if it's a secret/password
                          let lower = var_name.to_lowercase();
                          if lower.contains("password")
                              || lower.contains("secret")
                              || lower.contains("token")
                              || lower.contains("key")
                          {
                              setting.secret = Some(true);
                              setting.required = Some(true);
                          }

                          // Add default values for common settings
                          if let Some(defaults) = self.infer_defaults(var_name) {
                              setting.default = Some(defaults);
                          }

                          settings.push(setting);
                      }
                  }
              }
          }

          settings
      }

      fn infer_description(&self, var_name: &str) -> String {
          let descriptions = [
              ("DB_HOST", "Database server hostname"),
              ("DB_PORT", "Database server port"),
              ("DB_NAME", "Database name"),
              ("DB_USER", "Database username"),
              ("DB_PASSWORD", "Database password"),
              ("API_KEY", "API authentication key"),
              ("API_SECRET", "API secret"),
              ("API_URL", "API endpoint URL"),
              ("HOST", "Server hostname"),
              ("PORT", "Server port"),
          ];

          for (key, desc) in &descriptions {
              if *key == var_name {
                  return desc.to_string();
              }
          }

          var_name
              .split('_')
              .map(|word| {
                  let mut chars = word.chars();
                  match chars.next() {
                      None => String::new(),
                      Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                  }
              })
              .collect::<Vec<_>>()
              .join(" ")
      }

      fn infer_defaults(&self, var_name: &str) -> Option<String> {
          match var_name {
              "DB_HOST" => Some("localhost".to_string()),
              "DB_PORT" => Some("5432".to_string()),
              "HOST" => Some("localhost".to_string()),
              "PORT" => Some("8080".to_string()),
              "API_URL" => Some("https://api.example.com".to_string()),
              _ => None,
          }
      }

      async fn generate_gemini_context(&self) -> Result<String> {
          let frontmatter = self
              .metadata
              .source
              .frontmatter
              .as_ref()
              .ok_or_else(|| anyhow!("No frontmatter found"))?;

          let content = self
              .metadata
              .source
              .content
              .as_ref()
              .ok_or_else(|| anyhow!("No content found"))?;

          let mut gemini_content = format!(
              "# {} - Gemini CLI Extension\n\n",
              frontmatter.name
          );
          gemini_content.push_str(
              frontmatter
                  .description
                  .as_deref()
                  .unwrap_or(""),
          );
          gemini_content.push_str("\n\n## Quick Start\n\nAfter installation, you can use this extension by asking questions or giving commands naturally.\n\n");
          gemini_content.push_str(content);
          gemini_content.push_str("\n\n---\n\n");
          gemini_content.push_str("*This extension was converted from a Claude Code skill using [skill-porter](https://github.com/jduncan-rva/skill-porter)*\n");

          let output_path = self.output_path.join("GEMINI.md");
          fs::write(&output_path, gemini_content).await?;

          Ok(output_path.to_string_lossy().to_string())
      }

      async fn generate_commands(&self) -> Result<Vec<String>> {
          let mut generated_files = Vec::new();
          let commands_dir = self.output_path.join("commands");

          let subagents = self
              .metadata
              .source
              .subagents
              .as_ref()
              .map(|s| s.as_slice())
              .unwrap_or(&[]);
          let commands = self
              .metadata
              .source
              .commands
              .as_ref()
              .map(|c| c.as_slice())
              .unwrap_or(&[]);

          if subagents.is_empty() && commands.is_empty() {
              return Ok(generated_files);
          }

          fs::create_dir_all(&commands_dir).await?;

          // Convert Subagents -> Commands
          for agent in subagents {
              let toml_content = format!(
                  "description = \"Activate {} agent\"\n\n# Agent Persona: {}\n# Auto-generated from Claude Subagent\nprompt = \"\"\"\nYou are acting as the '{}' agent.\n{}\n\nUser Query: {{{{args}}}}\n\"\"\"\n",
                  agent.name,
                  agent.name,
                  agent.name,
                  agent.description.as_deref().unwrap_or("")
              );

              let file_path = commands_dir.join(format!("{}.toml", agent.name));
              fs::write(&file_path, toml_content).await?;
              generated_files.push(file_path.to_string_lossy().to_string());
          }

          // Convert Claude Commands -> Gemini Commands
          for cmd in commands {
              let frontmatter_regex = Regex::new(r"^---\n([\s\S]+?)\n---\n([\s\S]+)$")?;
              let mut description = format!("Custom command: {}", cmd.name);
              let mut prompt = cmd.content.clone();

              if let Some(caps) = frontmatter_regex.captures(&cmd.content) {
                  if let Ok(fm) = serde_yaml::from_str::<serde_yaml::Value>(caps.get(1).unwrap().as_str()) {
                      if let Some(desc) = fm.get("description") {
                          if let Some(desc_str) = desc.as_str() {
                              description = desc_str.to_string();
                          }
                      }
                  }
                  prompt = caps.get(2).unwrap().as_str().to_string();
              }

              // Convert arguments syntax
              prompt = prompt.replace("$ARGUMENTS", "{{args}}");
              let arg_regex = Regex::new(r"\$\d+")?;
              prompt = arg_regex.replace_all(&prompt, "{{args}}").to_string();

              let toml_content = format!(
                  "description = \"{}\"\n\nprompt = \"\"\"\n{}\n\"\"\"\n",
                  description, prompt.trim()
              );

              let file_path = commands_dir.join(format!("{}.toml", cmd.name));
              fs::write(&file_path, toml_content).await?;
              generated_files.push(file_path.to_string_lossy().to_string());
          }

          Ok(generated_files)
      }

      async fn ensure_shared_structure(&self) -> Result<()> {
          let shared_dir = self.output_path.join("shared");

          if fs::try_exists(&shared_dir).await.unwrap_or(false) {
              return Ok(());
          }

          fs::create_dir_all(&shared_dir).await?;

          let reference_content = r#"# Technical Reference

## Architecture
For detailed extension architecture, please refer to `docs/GEMINI_ARCHITECTURE.md` (in Gemini extensions) or the `SKILL.md` structure (in Claude Skills).

## Platform Differences
- **Commands:**
  - Gemini uses `commands/*.toml`
  - Claude uses `.claude/commands/*.md`
- **Agents:**
  - Gemini "Agents" are implemented as Custom Commands.
  - Claude "Subagents" are defined in `SKILL.md` frontmatter.
"#;

          fs::write(shared_dir.join("reference.md"), reference_content).await?;
          fs::write(
              shared_dir.join("examples.md"),
              "# Usage Examples\n\nComprehensive usage examples and tutorials.\n",
          )
          .await?;

          Ok(())
      }

      async fn inject_docs(&self) -> Result<()> {
          let docs_dir = self.output_path.join("docs");
          fs::create_dir_all(&docs_dir).await?;

          let arch_content = "# Gemini Architecture\n\nSee online documentation.";
          fs::write(docs_dir.join("GEMINI_ARCHITECTURE.md"), arch_content).await?;

          Ok(())
      }
  }

  // ============================================================================
  // Tests
  // ============================================================================

  #[cfg(test)]
  mod tests {
      use super::*;
      use std::fs;
      use tempfile::TempDir;

      #[tokio::test]
      async fn test_converter_initialization() {
          let source = "/tmp/source";
          let output = "/tmp/output";
          let converter = ClaudeToGeminiConverter::new(source, Some(output));

          assert_eq!(converter.source_path, PathBuf::from(source));
          assert_eq!(converter.output_path, PathBuf::from(output));
      }

      #[tokio::test]
      async fn test_infer_description() {
          let converter = ClaudeToGeminiConverter::new("/tmp", None);

          assert_eq!(
              converter.infer_description("DB_HOST"),
              "Database server hostname"
          );
          assert_eq!(
              converter.infer_description("API_KEY"),
              "API authentication key"
          );
          assert_eq!(
              converter.infer_description("CUSTOM_VAR"),
              "Custom Var"
          );
      }

      #[tokio::test]
      async fn test_infer_defaults() {
          let converter = ClaudeToGeminiConverter::new("/tmp", None);

          assert_eq!(
              converter.infer_defaults("DB_HOST"),
              Some("localhost".to_string())
          );
          assert_eq!(
              converter.infer_defaults("PORT"),
              Some("8080".to_string())
          );
          assert_eq!(converter.infer_defaults("UNKNOWN"), None);
      }

      #[tokio::test]
      async fn test_convert_allowed_tools_to_exclude() {
          let mut converter = ClaudeToGeminiConverter::new("/tmp", None);

          let allowed_tools = json!(["Read", "Write", "Bash"]);
          let excluded = converter.convert_allowed_tools_to_exclude(&allowed_tools);

          assert!(excluded.contains(&"Edit".to_string()));
          assert!(!excluded.contains(&"Read".to_string()));
      }

      #[tokio::test]
      async fn test_transform_mcp_servers() {
          let converter = ClaudeToGeminiConverter::new("/tmp", None);

          let mut mcp_servers = HashMap::new();
          let mut server = MCPServer {
              command: Some("node".to_string()),
              args: Some(vec!["server.js".to_string()]),
              env: None,
          };
          mcp_servers.insert("test-server".to_string(), server);

          let transformed = converter.transform_mcp_servers(&mcp_servers);

          assert!(transformed.contains_key("test-server"));
          if let Some(srv) = transformed.get("test-server") {
              if let Some(args) = &srv.args {
                  assert!(args[0].contains("${extensionPath}"));
              }
          }
      }

      #[tokio::test]
      async fn test_full_conversion_workflow() {
          let temp_dir = TempDir::new().unwrap();
          let source_path = temp_dir.path().join("source");
          let output_path = temp_dir.path().join("output");

          fs::create_dir_all(&source_path).unwrap();

          // Create a mock SKILL.md
          let skill_content = r#"---
  name: Test Skill
  description: A test skill
  subagents:
    - name: Agent1
      description: First agent
  ---
  # Content here
  "#;

          fs::write(source_path.join("SKILL.md"), skill_content).unwrap();

          let mut converter = ClaudeToGeminiConverter::new(&source_path, Some(&output_path));
          let result = converter.convert().await.unwrap();

          assert!(result.success);
          assert!(!result.files.is_empty());
      }

      #[test]
      fn test_frontmatter_parsing() {
          let yaml_str = r#"
  name: Test Skill
  description: A test skill
  allowed-tools:
    - Read
    - Write
  "#;

          let frontmatter: Frontmatter = serde_yaml::from_str(yaml_str).unwrap();
          assert_eq!(frontmatter.name, "Test Skill");
          assert_eq!(frontmatter.description, Some("A test skill".to_string()));
      }

      #[test]
      fn test_gemini_manifest_serialization() {
          let manifest = GeminiManifest {
              name: "Test Extension".to_string(),
              version: "1.0.0".to_string(),
              description: "Test description".to_string(),
              context_file_name: "GEMINI.md".to_string(),
              mcp_servers: None,
              settings: None,
              exclude_tools: None,
          };

          let json = serde_json::to_string_pretty(&manifest).unwrap();
          assert!(json.contains("\"name\": \"Test Extension\""));
          assert!(json.contains("\"version\": \"1.0.0\""));
      }

      #[test]
      fn test_setting_with_secret() {
          let setting = Setting {
              name: "API_KEY".to_string(),
              description: "API Key".to_string(),
              default: None,
              secret: Some(true),
              required: Some(true),
          };

          let json = serde_json::to_string(&setting).unwrap();
          assert!(json.contains("\"secret\": true"));
          assert!(json.contains("\"required\": true"));
      }
  }

  // ============================================================================
  // Main
  // ============================================================================

  #[tokio::main]
  async fn main() -> Result<()> {
      let args: Vec<String> = std::env::args().collect();

      if args.len() < 2 {
          eprintln!("Usage: {} <source_path> [output_path]", args[0]);
          std::process::exit(1);
      }

      let source_path = &args[1];
      let output_path = args.get(2).map(|s| s.as_str());

      let mut converter = ClaudeToGeminiConverter::new(source_path, output_path);
      let result = converter.convert().await?;

      println!("Conversion Result:");
      println!("  Success: {}", result.success);
      println!("  Files Generated: {}", result.files.len());
      for file in &result.files {
          println!("    - {}", file);
      }

      if !result.warnings.is_empty() {
          println!("  Warnings:");
          for warning in &result.warnings {
              println!("    - {}", warning);
          }
      }

      if !result.errors.is_empty() {
          println!("  Errors:");
          for error in &result.errors {
              println!("    - {}", error);
          }
      }

      Ok(())
  }