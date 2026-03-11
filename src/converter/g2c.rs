  use anyhow::{anyhow, Result};
  use regex::Regex;
  use serde::{Deserialize, Serialize};
  use serde_json::{json, Value};
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};
  use tokio::fs;

  // ============================================================================
  // Models
  // ============================================================================

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct GeminiManifest {
      pub name: String,
      pub version: Option<String>,
      pub description: String,
      #[serde(rename = "contextFileName")]
      pub context_file_name: Option<String>,
      #[serde(rename = "mcpServers")]
      pub mcp_servers: Option<HashMap<String, MCPServer>>,
      pub settings: Option<Vec<Setting>>,
      #[serde(rename = "excludeTools")]
      pub exclude_tools: Option<Vec<String>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MCPServer {
      pub command: Option<String>,
      pub args: Option<Vec<String>>,
      pub env: Option<HashMap<String, String>>,
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
  pub struct Command {
      pub name: String,
      pub content: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct ClaudeSkillFrontmatter {
      pub name: String,
      pub description: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      #[serde(rename = "allowed-tools")]
      pub allowed_tools: Option<Vec<String>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MarketplaceOwner {
      pub name: String,
      pub email: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MarketplaceMetadata {
      pub description: String,
      pub version: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PluginRepository {
      #[serde(rename = "type")]
      pub repo_type: String,
      pub url: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Plugin {
      pub name: String,
      pub description: String,
      pub source: String,
      pub strict: bool,
      pub author: String,
      pub repository: PluginRepository,
      pub license: String,
      pub keywords: Vec<String>,
      pub category: String,
      pub tags: Vec<String>,
      pub skills: Vec<String>,
      #[serde(skip_serializing_if = "Option::is_none")]
      #[serde(rename = "mcpServers")]
      pub mcp_servers: Option<HashMap<String, MCPServer>>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Marketplace {
      pub name: String,
      pub owner: MarketplaceOwner,
      pub metadata: MarketplaceMetadata,
      pub plugins: Vec<Plugin>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct MigrationInsight {
      #[serde(rename = "type")]
      pub insight_type: String,
      pub command: String,
      pub message: String,
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
      pub manifest: Option<GeminiManifest>,
      pub content: Option<String>,
      pub commands: Option<Vec<Command>>,
  }

  // ============================================================================
  // Main Converter
  // ============================================================================

  pub struct GeminiToClaudeConverter {
      source_path: PathBuf,
      output_path: PathBuf,
      metadata: Metadata,
  }

  impl GeminiToClaudeConverter {
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
                      manifest: None,
                      content: None,
                      commands: None,
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

          // Step 2: Extract metadata from Gemini extension
          if let Err(e) = self.extract_gemini_metadata().await {
              result.errors.push(format!("Failed to extract metadata: {}", e));
              return Ok(result);
          }

          // Step 3: Generate SKILL.md
          match self.generate_claude_skill().await {
              Ok(path) => result.files.push(path),
              Err(e) => result.errors.push(format!("Failed to generate SKILL.md: {}", e)),
          }

          // Step 4: Generate .claude-plugin/marketplace.json
          match self.generate_marketplace_json().await {
              Ok(path) => result.files.push(path),
              Err(e) => result.errors.push(format!("Failed to generate marketplace.json: {}", e)),
          }

          // Step 5: Generate Custom Commands
          match self.generate_claude_commands().await {
              Ok(paths) => result.files.extend(paths),
              Err(e) => result.errors.push(format!("Failed to generate commands: {}", e)),
          }

          // Step 6: Ensure shared directory structure
          if let Err(e) = self.ensure_shared_structure().await {
              result.warnings.push(format!("Failed to create shared structure: {}", e));
          }

          // Step 7: Generate Migration Insights
          if let Err(e) = self.generate_migration_insights().await {
              result.warnings.push(format!("Failed to generate insights: {}", e));
          }

          result.success = result.errors.is_empty();
          result.metadata = Some(self.metadata.clone());

          Ok(result)
      }

      // ========================================================================
      // Extraction Methods
      // ========================================================================

      async fn extract_gemini_metadata(&mut self) -> Result<()> {
          // Extract from gemini-extension.json
          let manifest_path = self.source_path.join("gemini-extension.json");
          let manifest_content = fs::read_to_string(&manifest_path).await?;
          let manifest: GeminiManifest = serde_json::from_str(&manifest_content)?;
          self.metadata.source.manifest = Some(manifest.clone());

          // Extract from GEMINI.md or custom context file
          let context_file_name = manifest
              .context_file_name
              .as_deref()
              .unwrap_or("GEMINI.md");
          let context_path = self.source_path.join(context_file_name);

          let content = match fs::read_to_string(&context_path).await {
              Ok(c) => Some(c),
              Err(_) => None,
          };
          self.metadata.source.content = content;

          // Extract commands if present
          let commands_dir = self.source_path.join("commands");
          let mut commands = Vec::new();

          if let Ok(mut entries) = fs::read_dir(&commands_dir).await {
              while let Some(entry) = entries.next_entry().await? {
                  let path = entry.path();
                  if path.extension().map_or(false, |ext| ext == "toml") {
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

          Ok(())
      }

      // ========================================================================
      // Generation Methods
      // ========================================================================

      async fn generate_claude_skill(&self) -> Result<String> {
          let manifest = self
              .metadata
              .source
              .manifest
              .as_ref()
              .ok_or_else(|| anyhow!("No manifest found"))?;

          let content = self.metadata.source.content.as_deref().unwrap_or("");

          // Build frontmatter
          let mut frontmatter = ClaudeSkillFrontmatter {
              name: manifest.name.clone(),
              description: manifest.description.clone(),
              allowed_tools: None,
          };

          // Convert excludeTools to allowed-tools
          if let Some(exclude_tools) = &manifest.exclude_tools {
              frontmatter.allowed_tools = Some(self.convert_exclude_to_allowed_tools(exclude_tools));
          }

          // Convert frontmatter to YAML
          let yaml_frontmatter = serde_yaml::to_string(&frontmatter)?;

          // Build SKILL.md content
          let mut skill_content = format!("---\n{}---\n\n", yaml_frontmatter);
          skill_content.push_str(&format!("# {} - Claude Code Skill\n\n", manifest.name));
          skill_content.push_str(&format!("{}\n\n", manifest.description));

          // Clean content (remove Gemini-specific headers)
          let mut clean_content = content.to_string();

          // Remove Gemini-specific headers
          let gemini_header_regex = Regex::new(r"^#\s+.+?\s+-\s+Gemini CLI Extension\n\n")?;
          clean_content = gemini_header_regex.replace(&clean_content, "").to_string();

          let quick_start_regex = Regex::new(r"##\s+Quick Start[\s\S]+?After installation.+?\n\n")?;
          clean_content = quick_start_regex.replace(&clean_content, "").to_string();

          let footer_regex = Regex::new(r"\n---\n\n\*This extension was converted.+?\*\n$")?;
          clean_content = footer_regex.replace(&clean_content, "").to_string();

          // Add environment variable configuration section if there are settings
          if let Some(settings) = &manifest.settings {
              if !settings.is_empty() {
                  skill_content.push_str("## Configuration\n\n");
                  skill_content.push_str("This skill requires the following environment variables:\n\n");

                  for setting in settings {
                      skill_content.push_str(&format!(
                          "- `{}`: {}",
                          setting.name, setting.description
                      ));

                      if let Some(ref default) = setting.default {
                          skill_content.push_str(&format!(" (default: {})", default));
                      }

                      if setting.required == Some(true) {
                          skill_content.push_str(" **(required)**");
                      }

                      skill_content.push_str("\n");
                  }

                  skill_content.push_str("\nSet these in your environment or Claude Code configuration.\n\n");
              }
          }

          // Add cleaned content
          if !clean_content.trim().is_empty() {
              skill_content.push_str(clean_content.trim());
              skill_content.push_str("\n\n");
          } else {
              skill_content.push_str(&format!(
                  "## Usage\n\nUse this skill when you need {}.\n\n",
                  manifest.description.to_lowercase()
              ));
          }

          // Add footer
          skill_content.push_str("---\n\n");
          skill_content.push_str("*This skill was converted from a Gemini CLI extension using [skill-porter](https://github.com/jduncan-rva/skill-porter)*\n");

          // Write to file
          let output_path = self.output_path.join("SKILL.md");
          fs::write(&output_path, skill_content).await?;

          Ok(output_path.to_string_lossy().to_string())
      }

      fn convert_exclude_to_allowed_tools(&self, exclude_tools: &[String]) -> Vec<String> {
          let all_tools = vec![
              "Read", "Write", "Edit", "Glob", "Grep", "Bash", "Task", "WebFetch", "WebSearch",
              "TodoWrite", "AskUserQuestion", "SlashCommand", "Skill", "NotebookEdit",
              "BashOutput", "KillShell",
          ];

          all_tools
              .iter()
              .filter(|tool| !exclude_tools.contains(&tool.to_string()))
              .map(|s| s.to_string())
              .collect()
      }

      async fn generate_marketplace_json(&self) -> Result<String> {
          let manifest = self
              .metadata
              .source
              .manifest
              .as_ref()
              .ok_or_else(|| anyhow!("No manifest found"))?;

          let keywords = self.extract_keywords(&manifest.description);

          let mut plugin = Plugin {
              name: manifest.name.clone(),
              description: manifest.description.clone(),
              source: ".".to_string(),
              strict: false,
              author: "Converted from Gemini".to_string(),
              repository: PluginRepository {
                  repo_type: "git".to_string(),
                  url: format!("https://github.com/user/{}", manifest.name),
              },
              license: "MIT".to_string(),
              keywords,
              category: "general".to_string(),
              tags: Vec::new(),
              skills: vec![".".to_string()],
              mcp_servers: None,
          };

          // Add MCP servers configuration if present
          if let Some(mcp_servers) = &manifest.mcp_servers {
              plugin.mcp_servers = Some(self.transform_mcp_servers_for_claude(
                  mcp_servers,
                  manifest.settings.as_deref(),
              ));
          }

          let marketplace = Marketplace {
              name: format!("{}-marketplace", manifest.name),
              owner: MarketplaceOwner {
                  name: "Skill Porter User".to_string(),
                  email: "user@example.com".to_string(),
              },
              metadata: MarketplaceMetadata {
                  description: manifest.description.clone(),
                  version: manifest.version.clone().unwrap_or_else(|| "1.0.0".to_string()),
              },
              plugins: vec![plugin],
          };

          // Create .claude-plugin directory
          let claude_plugin_dir = self.output_path.join(".claude-plugin");
          fs::create_dir_all(&claude_plugin_dir).await?;

          // Write to file
          let output_path = claude_plugin_dir.join("marketplace.json");
          let json_content = serde_json::to_string_pretty(&marketplace)?;
          fs::write(&output_path, json_content).await?;

          Ok(output_path.to_string_lossy().to_string())
      }

      async fn generate_claude_commands(&self) -> Result<Vec<String>> {
          let mut generated_files = Vec::new();
          let commands = self
              .metadata
              .source
              .commands
              .as_ref()
              .map(|c| c.as_slice())
              .unwrap_or(&[]);

          if commands.is_empty() {
              return Ok(generated_files);
          }

          let commands_dir = self.output_path.join(".claude").join("commands");
          fs::create_dir_all(&commands_dir).await?;

          let desc_regex = Regex::new(r#"description\s*=\s*"([^"]+)""#)?;
          let prompt_regex = Regex::new(r#"prompt\s*=\s*"""([\s\S]+?)"""#)?;

          for cmd in commands {
              let description = desc_regex
                  .captures(&cmd.content)
                  .and_then(|caps| caps.get(1))
                  .map(|m| m.as_str())
                  .unwrap_or(&format!("Run {}", cmd.name));

              let mut prompt = prompt_regex
                  .captures(&cmd.content)
                  .and_then(|caps| caps.get(1))
                  .map(|m| m.as_str())
                  .unwrap_or("")
                  .to_string();

              // Convert arguments syntax
              // Gemini: {{args}} -> Claude: $ARGUMENTS
              prompt = prompt.replace("{{args}}", "$ARGUMENTS");

              let md_content = format!(
                  "---\ndescription: {}\n---\n\n{}\n",
                  description,
                  prompt.trim()
              );

              let file_path = commands_dir.join(format!("{}.md", cmd.name));
              fs::write(&file_path, md_content).await?;
              generated_files.push(file_path.to_string_lossy().to_string());
          }

          Ok(generated_files)
      }

      async fn generate_migration_insights(&self) -> Result<()> {
          let commands = self
              .metadata
              .source
              .commands
              .as_ref()
              .map(|c| c.as_slice())
              .unwrap_or(&[]);

          let mut insights = Vec::new();
          let prompt_regex = Regex::new(r#"prompt\s*=\s*"""([\s\S]+?)"""#)?;
          let persona_regex = Regex::new(r"You are a|Act as|Your role is")?;

          // Heuristic checks
          for cmd in commands {
              if let Some(caps) = prompt_regex.captures(&cmd.content) {
                  let prompt = caps.get(1).unwrap().as_str();

                  // Check for Persona definition
                  if persona_regex.is_match(prompt) {
                      insights.push(MigrationInsight {
                          insight_type: "PERSONA_DETECTED".to_string(),
                          command: cmd.name.clone(),
                          message: format!(
                              "Command `/{0}` appears to define a persona. Consider moving this logic to `SKILL.md` instructions so Claude can adopt it automatically without a slash command.",
                              cmd.name
                          ),
                      });
                  }
              }
          }

          // Generate Report Content
          let mut content = String::from("# Migration Insights & Recommendations\n\n");
          content.push_str("Generated during conversion from Gemini to Claude.\n\n");

          if !insights.is_empty() {
              content.push_str("## 💡 Optimization Opportunities\n\n");
              content.push_str("While we successfully converted your commands to Claude Slash Commands, some might work better as native Skill instructions.\n\n");

              for insight in &insights {
                  content.push_str(&format!("### `/{}`\n", insight.command));
                  content.push_str(&format!("{}\n\n", insight.message));
              }

              content.push_str("## How to Apply\n");
              content.push_str("1. Open `SKILL.md`\n");
              content.push_str("2. Paste the prompt instructions into the main description area.\n");
              if let Some(first_insight) = insights.first() {
                  content.push_str(&format!(
                      "3. Delete `.claude/commands/{}.md` if you prefer automatic invocation.\n",
                      first_insight.command
                  ));
              }
          } else {
              content.push_str("✅ No specific architectural changes recommended. The direct conversion should work well.\n");
          }

          let shared_dir = self.output_path.join("shared");
          fs::create_dir_all(&shared_dir).await?;
          fs::write(shared_dir.join("MIGRATION_INSIGHTS.md"), content).await?;

          Ok(())
      }

      fn transform_mcp_servers_for_claude(
          &self,
          mcp_servers: &HashMap<String, MCPServer>,
          settings: Option<&[Setting]>,
      ) -> HashMap<String, MCPServer> {
          let mut transformed = HashMap::new();

          for (server_name, config) in mcp_servers {
              let mut new_config = config.clone();

              // Transform args to remove ${extensionPath}
              if let Some(args) = &config.args {
                  new_config.args = Some(
                      args.iter()
                          .map(|arg| arg.replace("${extensionPath}/", ""))
                          .collect(),
                  );
              }

              // Transform env to use ${VAR} pattern
              if let Some(env) = &config.env {
                  let mut new_env = HashMap::new();
                  for (key, value) in env {
                      new_env.insert(key.clone(), value.clone());
                  }
                  new_config.env = Some(new_env);
              }

              transformed.insert(server_name.clone(), new_config);
          }

          transformed
      }

      fn extract_keywords(&self, description: &str) -> Vec<String> {
          let common_words = [
              "the", "a", "an", "and", "or", "but", "for", "with", "to", "from", "in", "on",
          ];

          description
              .to_lowercase()
              .split_whitespace()
              .filter(|word| word.len() > 3 && !common_words.contains(&word))
              .take(5)
              .map(|s| s.to_string())
              .collect()
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
          let converter = GeminiToClaudeConverter::new(source, Some(output));

          assert_eq!(converter.source_path, PathBuf::from(source));
          assert_eq!(converter.output_path, PathBuf::from(output));
      }

      #[test]
      fn test_convert_exclude_to_allowed_tools() {
          let converter = GeminiToClaudeConverter::new("/tmp", None);
          let exclude_tools = vec!["Edit".to_string(), "Grep".to_string()];

          let allowed = converter.convert_exclude_to_allowed_tools(&exclude_tools);

          assert!(allowed.contains(&"Read".to_string()));
          assert!(allowed.contains(&"Write".to_string()));
          assert!(!allowed.contains(&"Edit".to_string()));
          assert!(!allowed.contains(&"Grep".to_string()));
      }

      #[test]
      fn test_extract_keywords() {
          let converter = GeminiToClaudeConverter::new("/tmp", None);
          let description = "This is a powerful tool for managing databases and files";

          let keywords = converter.extract_keywords(description);

          assert!(keywords.contains(&"powerful".to_string()));
          assert!(keywords.contains(&"managing".to_string()));
          assert!(!keywords.contains(&"is".to_string())); // Common word
      }

      #[test]
      fn test_marketplace_owner_serialization() {
          let owner = MarketplaceOwner {
              name: "Test User".to_string(),
              email: "test@example.com".to_string(),
          };

          let json = serde_json::to_string(&owner).unwrap();
          assert!(json.contains("\"name\": \"Test User\""));
          assert!(json.contains("\"email\": \"test@example.com\""));
      }

      #[test]
      fn test_plugin_serialization() {
          let plugin = Plugin {
              name: "Test Plugin".to_string(),
              description: "A test plugin".to_string(),
              source: ".".to_string(),
              strict: false,
              author: "Test Author".to_string(),
              repository: PluginRepository {
                  repo_type: "git".to_string(),
                  url: "https://github.com/test/repo".to_string(),
              },
              license: "MIT".to_string(),
              keywords: vec!["test".to_string()],
              category: "general".to_string(),
              tags: vec![],
              skills: vec![".".to_string()],
              mcp_servers: None,
          };

          let json = serde_json::to_string_pretty(&plugin).unwrap();
          assert!(json.contains("\"name\": \"Test Plugin\""));
          assert!(json.contains("\"license\": \"MIT\""));
      }

      #[tokio::test]
      async fn test_full_conversion_workflow() {
          let temp_dir = TempDir::new().unwrap();
          let source_path = temp_dir.path().join("source");
          let output_path = temp_dir.path().join("output");

          fs::create_dir_all(&source_path).unwrap();

          // Create mock gemini-extension.json
          let manifest = json!({
              "name": "Test Extension",
              "version": "1.0.0",
              "description": "A test extension",
              "contextFileName": "GEMINI.md",
              "excludeTools": ["Edit", "Grep"]
          });

          fs::write(
              source_path.join("gemini-extension.json"),
              serde_json::to_string_pretty(&manifest).unwrap(),
          )
          .unwrap();

          // Create mock GEMINI.md
          fs::write(
              source_path.join("GEMINI.md"),
              "# Test Extension - Gemini CLI Extension\n\nTest content\n",
          )
          .unwrap();

          let mut converter = GeminiToClaudeConverter::new(&source_path, Some(&output_path));
          let result = converter.convert().await.unwrap();

          assert!(result.success);
          assert!(!result.files.is_empty());

          // Verify SKILL.md was created
          let skill_path = output_path.join("SKILL.md");
          assert!(skill_path.exists());

          // Verify marketplace.json was created
          let marketplace_path = output_path.join(".claude-plugin").join("marketplace.json");
          assert!(marketplace_path.exists());
      }

      #[tokio::test]
      async fn test_command_conversion() {
          let temp_dir = TempDir::new().unwrap();
          let source_path = temp_dir.path().join("source");
          let output_path = temp_dir.path().join("output");

          fs::create_dir_all(&source_path).unwrap();
          fs::create_dir_all(source_path.join("commands")).unwrap();

          // Create mock gemini-extension.json
          let manifest = json!({
              "name": "Test Extension",
              "description": "Test",
              "contextFileName": "GEMINI.md"
          });

          fs::write(
              source_path.join("gemini-extension.json"),
              serde_json::to_string_pretty(&manifest).unwrap(),
          )
          .unwrap();

          fs::write(source_path.join("GEMINI.md"), "Test").unwrap();

          // Create mock command
          let command_content = r#"description = "Test Command"

prompt = """
You are a test assistant.
User Query: {{args}}
"""
"#;

          fs::write(source_path.join("commands").join("test.toml"), command_content).unwrap();

          let mut converter = GeminiToClaudeConverter::new(&source_path, Some(&output_path));
          let result = converter.convert().await.unwrap();

          assert!(result.success);

          // Verify command was converted
          let cmd_path = output_path.join(".claude").join("commands").join("test.md");
          assert!(cmd_path.exists());

          let cmd_content = fs::read_to_string(cmd_path).unwrap();
          assert!(cmd_content.contains("$ARGUMENTS")); // Should convert {{args}} to $ARGUMENTS
      }

      #[test]
      fn test_gemini_manifest_deserialization() {
          let json_str = r#"{
              "name": "Test Extension",
              "version": "1.0.0",
              "description": "Test description",
              "contextFileName": "GEMINI.md",
              "excludeTools": ["Edit", "Grep"]
          }"#;

          let manifest: GeminiManifest = serde_json::from_str(json_str).unwrap();
          assert_eq!(manifest.name, "Test Extension");
          assert_eq!(manifest.version, Some("1.0.0".to_string()));
          assert_eq!(manifest.exclude_tools, Some(vec!["Edit".to_string(), "Grep".to_string()]));
      }

      #[test]
      fn test_migration_insight_serialization() {
          let insight = MigrationInsight {
              insight_type: "PERSONA_DETECTED".to_string(),
              command: "test_cmd".to_string(),
              message: "Test message".to_string(),
          };

          let json = serde_json::to_string(&insight).unwrap();
          assert!(json.contains("\"type\": \"PERSONA_DETECTED\""));
          assert!(json.contains("\"command\": \"test_cmd\""));
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

      let mut converter = GeminiToClaudeConverter::new(source_path, output_path);
      let result = converter.convert().await?;

      println!("Conversion Result:");
      println!("  Success: {}", result.success);
      println!("  Files Generated: {}", result.files.len());
      for file in &result.files {
          println!("    - {}", file);
      }

      if !result.warnings.is_empty() {
          println!("\n  Warnings:");
          for warning in &result.warnings {
              println!("    - {}", warning);
          }
      }

      if !result.errors.is_empty() {
          println!("\n  Errors:");
          for error in &result.errors {
              println!("    - {}", error);
          }
      }

      Ok(())
  }