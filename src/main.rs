use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Select};
use dirs::home_dir;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::symlink;

static CHECKMARK: Emoji<'_, '_> = Emoji("✓  ", "✓ ");
static CROSS: Emoji<'_, '_> = Emoji("✗  ", "✗ ");
static WARN: Emoji<'_, '_> = Emoji("⚠  ", "!");
static INFO: Emoji<'_, '_> = Emoji("ℹ  ", "i");

const BANNER: &str = r"
██████╗ ██╗      ██╗ ██╗███╗   ██╗██╗  ██╗
██╔══██╗██║      ██║███║████╗  ██║██║ ██╔╝
██████╔╝██║      ██║╚██║██╔██╗ ██║█████╔╝ 
██╔══██╗██║      ██║ ██║██║╚██╗██║██╔═██╗ 
██████╔╝███████╗ ██║ ██║██║ ╚████║██║  ██╗
╚═════╝ ╚══════╝ ╚═╝ ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
";

const TAGLINE: &str = "Bl1nk Team Kit - Spec-Driven Development Toolkit";

const ONBOARDING_TEMPLATE: &str = include_str!("templates/onboarding-template.md");

/* ================= AGENT CONFIG (for init) ================= */

#[derive(Debug, Clone)]
struct AgentConfig {
    name: &'static str,
    folder: &'static str,
    install_url: Option<&'static str>,
    requires_cli: bool,
}

macro_rules! agent_map {
    { $($key:expr => $name:expr, $folder:expr, $install_url:expr, $requires_cli:expr),* $(,)? } => {
        {
            let mut m = HashMap::new();
            $(
                m.insert($key, AgentConfig {
                    name: $name,
                    folder: $folder,
                    install_url: $install_url,
                    requires_cli: $requires_cli,
                });
            )*
            m
        }
    };
}

lazy_static::lazy_static! {
    static ref AGENT_CONFIG: HashMap<&'static str, AgentConfig> = agent_map! {
        "copilot" => "GitHub Copilot", ".github/", None, false,
        "claude" => "Claude Code", ".claude/", Some("https://docs.anthropic.com/en/docs/claude-code/setup"), true,
        "gemini" => "Gemini CLI", ".gemini/", Some("https://github.com/google-gemini/gemini-cli"), true,
        "cursor-agent" => "Cursor", ".cursor/", None, false,
        "qwen" => "Qwen Code", ".qwen/", Some("https://github.com/QwenLM/qwen-code"), true,
        "opencode" => "opencode", ".opencode/", Some("https://opencode.ai"), true,
        "codex" => "Codex CLI", ".codex/", Some("https://github.com/openai/codex"), true,
        "windsurf" => "Windsurf", ".windsurf/", None, false,
        "kilocode" => "Kilo Code", ".kilocode/", None, false,
        "auggie" => "Auggie CLI", ".augment/", Some("https://docs.augmentcode.com/cli/setup-auggie/install-auggie-cli"), true,
        "codebuddy" => "CodeBuddy", ".codebuddy/", Some("https://www.codebuddy.ai/cli"), true,
        "qoder" => "Qoder CLI", ".qoder/", Some("https://qoder.com/cli"), true,
        "roo" => "Roo Code", ".roo/", None, false,
        "q" => "Amazon Q Developer CLI", ".amazonq/", Some("https://aws.amazon.com/developer/learning/q-developer-cli/"), true,
        "amp" => "Amp", ".agents/", Some("https://ampcode.com/manual#install"), true,
        "shai" => "SHAI", ".shai/", Some("https://github.com/ovh/shai"), true,
        "bob" => "IBM Bob", ".bob/", None, false,
    };
}

const SCRIPT_TYPE_CHOICES: &[(&str, &str)] = &[
    ("sh", "POSIX Shell (bash/zsh)"),
    ("ps", "PowerShell"),
];

/* ================= CLI ================= */

#[derive(Parser)]
#[command(name = "bl")]
#[command(about = "Bl1nk CLI - Project initialization and skill management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Bl1nk project from the latest template
    Init {
        /// Name for your new project directory (optional if using --here)
        project_name: Option<String>,

        /// AI assistant to use (claude, gemini, copilot, etc.)
        #[arg(long)]
        ai: Option<String>,

        /// Script type to use: sh or ps
        #[arg(long)]
        script: Option<String>,

        /// Skip checks for AI agent tools like Claude Code
        #[arg(long)]
        ignore_agent_tools: bool,

        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,

        /// Initialize project in the current directory instead of creating a new one
        #[arg(long)]
        here: bool,

        /// Force merge/overwrite when using --here (skip confirmation)
        #[arg(long)]
        force: bool,

        /// Skip SSL/TLS verification (not recommended)
        #[arg(long)]
        skip_tls: bool,

        /// Show verbose diagnostic output for network and extraction failures
        #[arg(long)]
        debug: bool,

        /// GitHub token to use for API requests (or set GH_TOKEN or GITHUB_TOKEN env var)
        #[arg(long)]
        github_token: Option<String>,
    },

    /// Generate an analysis template for an existing project
    Onboard,

    /// Check that all required tools are installed
    Check,

    /// Display version and system information
    Version,

    /// Install a skill for an agent
    Install {
        agent: String,

        #[arg(long)]
        repo: Option<String>,

        #[arg(long)]
        url: Option<String>,

        #[arg(long)]
        path: String,

        #[arg(long, default_value = "main")]
        reference: String,
    },

    /// Uninstall a skill from an agent
    Uninstall {
        agent: String,
        skill: String,
    },

    /// List all globally installed skills
    List,

    /// List all agents that have a skills directory
    Agents,
}

/* ================= MAIN ================= */

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            project_name,
            ai,
            script,
            ignore_agent_tools,
            no_git,
            here,
            force,
            skip_tls,
            debug,
            github_token,
        } => cmd_init(
            project_name,
            ai,
            script,
            ignore_agent_tools,
            no_git,
            here,
            force,
            skip_tls,
            debug,
            github_token,
        ),
        Commands::Onboard => cmd_onboard(),
        Commands::Check => cmd_check(),
        Commands::Version => cmd_version(),
        Commands::Install {
            agent,
            repo,
            url,
            path,
            reference,
        } => cmd_install(agent, repo, url, path, reference),
        Commands::Uninstall { agent, skill } => cmd_uninstall(agent, skill),
        Commands::List => cmd_list(),
        Commands::Agents => cmd_agents(),
    }
}

/* ================= BANNER ================= */

/// Returns the banner with spaces between each character for better readability.
fn spaced_banner() -> String {
    let lines: Vec<&str> = BANNER.trim().split('\n').collect();
    let mut result = String::new();
    for line in lines {
        for c in line.chars() {
            result.push(c);
            result.push(' ');
        }
        result.push('\n');
    }
    result
}

fn show_banner() {
    let banner = spaced_banner();
    let lines: Vec<&str> = banner.trim().split('\n').collect();
    let colors = [
        console::Color::Magenta,
        console::Color::BrightMagenta,
        console::Color::Cyan,
        console::Color::BrightPurple,
    ];
    for (i, line) in lines.iter().enumerate() {
        let color = colors[i % colors.len()];
        println!("{}", style(*line).color(color));
    }
    println!("{}", style(TAGLINE).italic().yellow());
    println!();
}

/* ================= INIT COMMAND ================= */

fn cmd_init(
    project_name: Option<String>,
    ai: Option<String>,
    script: Option<String>,
    ignore_agent_tools: bool,
    no_git: bool,
    here: bool,
    force: bool,
    skip_tls: bool,
    debug: bool,
    github_token: Option<String>,
) -> Result<()> {
    show_banner();

    // Handle project path
    let (project_path, is_current_dir) = if here || project_name.as_deref() == Some(".") {
        (std::env::current_dir()?, true)
    } else if let Some(name) = project_name {
        (std::env::current_dir()?.join(name), false)
    } else {
        anyhow::bail!("Must specify either a project name, use '.' for current directory, or use --here flag");
    };

    if !is_current_dir && project_path.exists() {
        anyhow::bail!(
            "Directory '{}' already exists. Please choose a different name or remove it.",
            project_path.display()
        );
    }

    if is_current_dir {
        let entries = fs::read_dir(&project_path)?.count();
        if entries > 0 {
            println!(
                "{} Current directory is not empty ({} items). Template files will be merged.",
                WARN, entries
            );
            if !force {
                if !dialoguer::Confirm::new()
                    .with_prompt("Do you want to continue?")
                    .interact()?
                {
                    println!("{} Operation cancelled", INFO);
                    return Ok(());
                }
            }
        }
    }

    // Print setup info
    println!("{} Bl1nk Project Setup", style("▶").magenta());
    println!("  Project:      {}", style(project_path.file_name().unwrap_or_default().to_string_lossy()).green());
    println!("  Working path: {}", style(std::env::current_dir()?.display()).dim());
    if !is_current_dir {
        println!("  Target path:  {}", style(project_path.display()).dim());
    }
    println!();

    // Check git if needed
    let git_available = if !no_git { check_tool("git") } else { false };
    if !no_git && !git_available {
        println!("{} Git not found - will skip repository initialization", WARN);
    }

    // AI assistant selection
    let selected_ai = match ai {
        Some(a) if AGENT_CONFIG.contains_key(a.as_str()) => a,
        Some(a) => anyhow::bail!("Invalid AI assistant '{}'. Choose from: {}", a, AGENT_CONFIG.keys().cloned().collect::<Vec<_>>().join(", ")),
        None => {
            let agents: Vec<&str> = AGENT_CONFIG.keys().copied().collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose your AI assistant")
                .default(0)
                .items(&agents)
                .interact()?;
            agents[selection].to_string()
        }
    };
    let agent_cfg = AGENT_CONFIG.get(selected_ai.as_str()).unwrap();

    // Check agent CLI if required
    if !ignore_agent_tools && agent_cfg.requires_cli && !check_tool(&selected_ai) {
        let install_url = agent_cfg.install_url.unwrap_or("(no URL)");
        anyhow::bail!(
            "{} not found. Install from: {}\n{} is required for this project type.\nTip: Use --ignore-agent-tools to skip this check.",
            agent_cfg.name, install_url, agent_cfg.name
        );
    }

    // Script type selection
    let selected_script = match script {
        Some(s) if SCRIPT_TYPE_CHOICES.iter().any(|(k, _)| *k == s) => s,
        Some(s) => anyhow::bail!("Invalid script type '{}'. Choose from: sh, ps", s),
        None => {
            let default = if cfg!(windows) { "ps" } else { "sh" };
            let items: Vec<&str> = SCRIPT_TYPE_CHOICES.iter().map(|(k, _)| *k).collect();
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose script type")
                .default(items.iter().position(|&x| x == default).unwrap_or(0))
                .items(&items)
                .interact()?;
            items[selection].to_string()
        }
    };

    println!("{} AI assistant: {}", CHECKMARK, style(&selected_ai).magenta());
    println!("{} Script type:  {}", CHECKMARK, style(&selected_script).magenta());
    println!();

    // Download and extract template
    let github_token = github_token.or_else(|| env::var("GH_TOKEN").ok()).or_else(|| env::var("GITHUB_TOKEN").ok());
    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(skip_tls)
        .build()?;

    // Show progress steps
    println!("{} Fetch latest release...", INFO);
    let (zip_path, release_tag, asset_name) = download_template(&client, &selected_ai, &selected_script, &github_token, debug)?;
    println!("{} Downloaded: {}", CHECKMARK, asset_name);

    println!("{} Extracting template...", INFO);
    extract_template(&zip_path, &project_path, is_current_dir)?;
    println!("{} Extracted", CHECKMARK);

    // Set executable permissions on .sh scripts (Unix only)
    if !cfg!(windows) {
        set_executable_permissions(&project_path)?;
    }

    // Git init
    if !no_git && git_available && !is_git_repo(&project_path)? {
        println!("{} Initializing git repository...", INFO);
        init_git_repo(&project_path)?;
        println!("{} Git repository initialized", CHECKMARK);
    } else if !no_git && git_available {
        println!("{} Git repository already exists", INFO);
    } else if !no_git {
        println!("{} Git not available, skipping", WARN);
    } else {
        println!("{} Git init skipped (--no-git)", INFO);
    }

    // Clean up zip
    let _ = fs::remove_file(&zip_path);

    println!("\n{} Project ready.", style("✔").green());

    // Security notice for agent folder
    if let Some(cfg) = AGENT_CONFIG.get(selected_ai.as_str()) {
        println!();
        println!("{} Agent Folder Security", style("⚠").yellow());
        println!("   Some agents may store credentials in {}.", style(cfg.folder).cyan());
        println!("   Consider adding it to .gitignore to prevent leakage.");
    }

    // Next steps
    println!();
    println!("{} Next Steps", style("▶").magenta());
    if !is_current_dir {
        println!("1. Go to the project folder: cd {}", style(project_path.display()).cyan());
    } else {
        println!("1. You're already in the project directory!");
    }
    println!("2. Start using Bl1nk CLI commands:");
    println!("   2.1 {} - Establish project principles", style("bl: constitution").cyan());
    println!("   2.2 {} - Create baseline specification", style("bl: specify").cyan());
    println!("   2.3 {} - Create implementation plan", style("bl: plan").cyan());
    println!("   2.4 {} - Generate actionable tasks", style("bl: tasks").cyan());
    println!("   2.5 {} - Execute implementation", style("bl: implement").cyan());

    // Enhancement commands
    println!();
    println!("{} Enhancement Commands", style("▶").magenta());
    println!("   Optional commands to improve quality:");
    println!("   ○ {} - Ask clarifying questions", style("bl: clarify").cyan());
    println!("   ○ {} - Consistency report", style("bl: analyze").cyan());
    println!("   ○ {} - Quality checklists", style("bl: checklist").cyan());

    Ok(())
}

/* ================= ONBOARD COMMAND ================= */

fn cmd_onboard() -> Result<()> {
    show_banner();
    println!("{} Generating onboarding template...", INFO);

    let output_path = std::env::current_dir()?.join("000-onboarding-analysis.md");
    if output_path.exists() {
        if !dialoguer::Confirm::new()
            .with_prompt("File already exists. Overwrite?")
            .interact()?
        {
            println!("{} Operation cancelled", INFO);
            return Ok(());
        }
    }

    fs::write(&output_path, ONBOARDING_TEMPLATE)?;
    println!("{} Created {}", CHECKMARK, style(output_path.display()).green());
    println!("\nNext steps:");
    println!("1. Open the file and follow the instructions for the agent.");
    println!("2. After analysis, start creating specs with bl: specify.");
    Ok(())
}

/* ================= CHECK COMMAND ================= */

fn cmd_check() -> Result<()> {
    show_banner();
    println!("{} Checking for installed tools...", style("▶").magenta());
    println!();

    let git = check_tool("git");
    println!("  {} Git: {}", if git { CHECKMARK } else { CROSS }, if git { "available" } else { "not found" });

    for (key, cfg) in AGENT_CONFIG.iter() {
        if cfg.requires_cli {
            let found = check_tool(key);
            println!("  {} {}: {}", if found { CHECKMARK } else { CROSS }, cfg.name, if found { "available" } else { "not found" });
        } else {
            println!("  {} {}: {} (IDE-based)", INFO, cfg.name, style("no CLI check").dim());
        }
    }

    let code = check_tool("code");
    println!("  {} Visual Studio Code: {}", if code { CHECKMARK } else { CROSS }, if code { "available" } else { "not found" });

    let code_insiders = check_tool("code-insiders");
    println!("  {} VS Code Insiders: {}", if code_insiders { CHECKMARK } else { CROSS }, if code_insiders { "available" } else { "not found" });

    println!();
    println!("{} Bl1nk CLI is ready to use!", style("✔").green());
    Ok(())
}

/* ================= VERSION COMMAND ================= */

fn cmd_version() -> Result<()> {
    show_banner();

    let cli_version = env!("CARGO_PKG_VERSION");
    let (template_version, release_date) = get_latest_template_version()?;

    println!("CLI Version:      {}", style(cli_version).magenta());
    println!("Template Version: {}", style(template_version).magenta());
    println!("Released:         {}", style(release_date).magenta());
    println!();
    println!("Platform:         {}", style(std::env::consts::OS).cyan());
    println!("Architecture:     {}", style(std::env::consts::ARCH).cyan());

    Ok(())
}

/* ================= SKILL INSTALL COMMAND ================= */

fn cmd_install(
    agent: String,
    repo: Option<String>,
    url: Option<String>,
    skill_path: String,
    reference: String,
) -> Result<()> {
    let (owner, repository) = resolve_source(repo, url)?;
    validate_relative_path(&skill_path)?;

    let tmp = tempdir()?;
    let repo_root = download_repo(&owner, &repository, &reference, tmp.path())?;

    let skill_src = repo_root.join(&skill_path);
    validate_skill(&skill_src)?;

    let skill_name = Path::new(&skill_path)
        .file_name()
        .context("Invalid skill path")?
        .to_string_lossy()
        .to_string();

    validate_skill_name(&skill_name)?;

    // Move into global store
    let global_root = global_dir();
    fs::create_dir_all(&global_root)?;
    let global_dest = global_root.join(&skill_name);

    if global_dest.exists() {
        bail!("Skill already exists in global store");
    }

    fs::rename(&skill_src, &global_dest)?;

    // Link into agent
    let agent_root = agent_dir(&agent);
    fs::create_dir_all(&agent_root)?;
    let link_path = agent_root.join(&skill_name);

    if link_path.exists() {
        fs::remove_file(&link_path)?;
    }

    #[cfg(unix)]
    symlink(&global_dest, &link_path)?;
    #[cfg(windows)]
    {
        // On Windows, create a junction or directory symlink? For simplicity, copy.
        fs::copy(&global_dest, &link_path)?;
    }

    println!("Installed {skill_name} for {agent}");

    Ok(())
}

/* ================= SKILL UNINSTALL COMMAND ================= */

fn cmd_uninstall(agent: String, skill: String) -> Result<()> {
    let path = agent_dir(&agent).join(skill);
    if path.exists() {
        fs::remove_file(path)?;
        println!("Removed");
    } else {
        println!("Skill not found");
    }
    Ok(())
}

/* ================= SKILL LIST COMMAND ================= */

fn cmd_list() -> Result<()> {
    let dir = global_dir();
    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            println!("{}", entry?.file_name().to_string_lossy());
        }
    }
    Ok(())
}

/* ================= SKILL AGENTS COMMAND ================= */

fn cmd_agents() -> Result<()> {
    let home = home_dir().unwrap();
    for entry in fs::read_dir(home)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && entry.path().join("skills").exists() {
            println!("{}", &name[1..]);
        }
    }
    Ok(())
}

/* ================= HELPER FUNCTIONS (Project Init) ================= */

fn check_tool(tool: &str) -> bool {
    if tool == "claude" {
        let claude_local = home_dir()
            .map(|p| p.join(".claude").join("local").join("claude"))
            .filter(|p| p.exists() && p.is_file());
        if claude_local.is_some() {
            return true;
        }
    }
    which::which(tool).is_ok()
}

fn is_git_repo(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(true),
        _ => Ok(false),
    }
}

fn init_git_repo(path: &Path) -> Result<()> {
    Command::new("git")
        .arg("init")
        .current_dir(path)
        .status()
        .context("git init failed")?;
    Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(path)
        .status()
        .context("git add failed")?;
    Command::new("git")
        .args(&["commit", "-m", "Initial commit from Bl1nk template"])
        .current_dir(path)
        .status()
        .context("git commit failed")?;
    Ok(())
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let scripts_dir = path.join(".bl1nk").join("scripts");
    if !scripts_dir.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&scripts_dir) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "sh") {
            let mut perms = fs::metadata(entry.path())?.permissions();
            let mode = perms.mode();
            let new_mode = mode
                | (if mode & 0o400 != 0 { 0o100 } else { 0 })
                | (if mode & 0o040 != 0 { 0o010 } else { 0 })
                | (if mode & 0o004 != 0 { 0o001 } else { 0 });
            if new_mode != mode {
                perms.set_mode(new_mode | 0o100);
                fs::set_permissions(entry.path(), perms)?;
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn set_executable_permissions(_path: &Path) -> Result<()> {
    // Windows doesn't use executable bits
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    published_at: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
}

fn download_template(
    client: &Client,
    ai: &str,
    script_type: &str,
    github_token: &Option<String>,
    debug: bool,
) -> Result<(PathBuf, String, String)> {
    // Changed to bl1nk-bot/skill-cli as requested
    let repo_owner = "bl1nk-bot";
    let repo_name = "skill-cli";
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", repo_owner, repo_name);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "bl1nk-cli/rust".parse().unwrap());
    if let Some(token) = github_token {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let response = client
        .get(&api_url)
        .headers(headers.clone())
        .send()
        .context("Failed to fetch latest release")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        if debug {
            eprintln!("GitHub API error {}: {}", status, text);
        }
        anyhow::bail!("GitHub API returned status {}", status);
    }

    let release: Release = response.json()?;
    let pattern = format!("bl1nk-template-{}-{}", ai, script_type);
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(&pattern) && a.name.ends_with(".zip"))
        .ok_or_else(|| anyhow!("No matching asset found for pattern '{}'", pattern))?;

    println!("  Found: {} ({} bytes)", asset.name, asset.size);

    // Download the zip
    let response = client
        .get(&asset.browser_download_url)
        .headers(headers)
        .send()
        .context("Failed to download template")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status {}", response.status());
    }

    let total_size = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg} {bar:40.magenta/blue} {bytes}/{total_bytes} ({eta})")?
        .progress_chars("=>-"));

    let mut data = Vec::new();
    let mut stream = response;
    let mut downloaded = 0;
    while let Some(chunk) = stream.chunk().context("Error reading download stream")? {
        data.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    pb.finish_and_clear();

    let zip_path = tempdir()?.path().join(&asset.name);
    fs::write(&zip_path, data)?;

    Ok((zip_path, release.tag_name, asset.name.clone()))
}

fn extract_template(zip_path: &Path, dest: &Path, flatten_nested: bool) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    if flatten_nested {
        // Extract to temp, then flatten
        let temp_dir = tempdir()?;
        archive.extract(temp_dir.path())?;

        let extracted = temp_dir.path();
        let items: Vec<PathBuf> = fs::read_dir(extracted)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();

        let source = if items.len() == 1 && items[0].is_dir() {
            items[0].clone()
        } else {
            extracted.to_path_buf()
        };

        copy_dir_all(&source, dest)?;
    } else {
        archive.extract(dest)?;

        // If extracted single top-level directory, move its contents up
        let items: Vec<PathBuf> = fs::read_dir(dest)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();

        if items.len() == 1 && items[0].is_dir() {
            let nested = items[0].clone();
            let temp_move = dest.parent().unwrap().join(format!("{}_temp", dest.file_name().unwrap().to_string_lossy()));
            fs::rename(&nested, &temp_move)?;
            fs::remove_dir(dest)?;
            fs::rename(&temp_move, dest)?;
        }
    }

    // Merge .vscode/settings.json if present
    let vscode_settings_src = dest.join(".vscode").join("settings.json");
    if vscode_settings_src.exists() {
        let dest_settings = dest.join(".vscode").join("settings.json");
        if dest_settings.exists() {
            merge_json_files(&dest_settings, &vscode_settings_src)?;
            fs::remove_file(&vscode_settings_src)?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn merge_json_files(existing: &Path, new: &Path) -> Result<()> {
    let existing_content: serde_json::Value = {
        let mut f = File::open(existing)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        serde_json::from_str(&s)?
    };

    let new_content: serde_json::Value = {
        let mut f = File::open(new)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        serde_json::from_str(&s)?
    };

    fn deep_merge(a: &mut serde_json::Value, b: &serde_json::Value) {
        match (a, b) {
            (serde_json::Value::Object(ref mut a_obj), serde_json::Value::Object(b_obj)) => {
                for (k, v) in b_obj {
                    if let Some(a_val) = a_obj.get_mut(k) {
                        deep_merge(a_val, v);
                    } else {
                        a_obj.insert(k.clone(), v.clone());
                    }
                }
            }
            (a, b) => *a = b.clone(),
        }
    }

    let mut merged = existing_content;
    deep_merge(&mut merged, &new_content);

    let mut f = File::create(existing)?;
    f.write_all(serde_json::to_string_pretty(&merged)?.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

fn get_latest_template_version() -> Result<(String, String)> {
    let client = reqwest::blocking::Client::new();
    let url = "https://api.github.com/repos/bl1nk-bot/skill-cli/releases/latest";
    let response = client
        .get(url)
        .header("User-Agent", "bl1nk-cli/rust")
        .send()?;

    if response.status().is_success() {
        let release: Release = response.json()?;
        let tag = release.tag_name.trim_start_matches('v').to_string();
        let date = chrono::DateTime::parse_from_rfc3339(&release.published_at)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or(release.published_at);
        Ok((tag, date))
    } else {
        Ok(("unknown".to_string(), "unknown".to_string()))
    }
}

/* ================= HELPER FUNCTIONS (Skills) ================= */

fn global_dir() -> PathBuf {
    home_dir().unwrap().join(".agents/skills")
}

fn agent_dir(agent: &str) -> PathBuf {
    home_dir().unwrap().join(format!(".{}/skills", agent))
}

fn download_repo(
    owner: &str,
    repo: &str,
    reference: &str,
    dest: &Path,
) -> Result<PathBuf> {
    let url = format!("https://codeload.github.com/{}/{}/zip/{}", owner, repo, reference);

    let response = reqwest::blocking::get(url)?.bytes()?;
    let reader = Cursor::new(response.to_vec());
    let mut archive = ZipArchive::new(reader)?;

    safe_extract(&mut archive, dest)?;

    let first = fs::read_dir(dest)?
        .next()
        .context("Empty archive")??
        .path();

    Ok(first)
}

pub fn safe_extract<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    dest: &Path,
) -> Result<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest.join(file.name());

        if !outpath.starts_with(dest) {
            bail!("Archive contains invalid path");
        }

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<()> {
    if Path::new(path).is_absolute() || path.contains("..") {
        bail!("Invalid skill path");
    }
    Ok(())
}

fn validate_skill(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("Skill directory not found");
    }

    if !path.join("SKILL.md").exists() {
        bail!("SKILL.md not found in skill directory");
    }

    Ok(())
}

fn validate_skill_name(name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        bail!("Invalid skill name");
    }
    Ok(())
}

fn resolve_source(
    repo: Option<String>,
    url: Option<String>,
) -> Result<(String, String)> {
    if let Some(repo) = repo {
        let parts: Vec<_> = repo.split('/').collect();
        if parts.len() != 2 {
            bail!("Repo must be owner/repo");
        }
        return Ok((parts[0].to_string(), parts[1].to_string()));
    }

    if let Some(url) = url {
        let parts: Vec<_> = url.split('/').collect();
        if parts.len() < 5 {
            bail!("Invalid GitHub URL");
        }
        return Ok((parts[3].to_string(), parts[4].to_string()));
    }

    bail!("Provide --repo or --url");
}