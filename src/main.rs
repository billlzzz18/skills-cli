use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use reqwest::blocking::get;
use std::fs;
use std::io::{Cursor, Read, Seek};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use zip::ZipArchive;

/* ================= CLI ================= */

#[derive(Parser)]
#[command(name = "skills")]
#[command(about = "Universal Agent Skill Installer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    Uninstall {
        agent: String,
        skill: String,
    },

    List,

    Agents,
}

/* ================= MAIN ================= */

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            agent,
            repo,
            url,
            path,
            reference,
        } => install(agent, repo, url, path, reference),

        Commands::Uninstall { agent, skill } => uninstall(agent, skill),

        Commands::List => list_global(),

        Commands::Agents => list_agents(),
    }
}

/* ================= PATHS ================= */

fn global_dir() -> PathBuf {
    home_dir().unwrap().join(".agents/skills")
}

fn agent_dir(agent: &str) -> PathBuf {
    home_dir().unwrap().join(format!(".{agent}/skills"))
}

/* ================= INSTALL ================= */

fn install(
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

    symlink(&global_dest, &link_path)?;

    println!("Installed {skill_name} for {agent}");

    Ok(())
}

/* ================= DOWNLOAD ================= */

fn download_repo(
    owner: &str,
    repo: &str,
    reference: &str,
    dest: &Path,
) -> Result<PathBuf> {
    let url =
        format!("https://codeload.github.com/{owner}/{repo}/zip/{reference}");

    let response = get(url)?.bytes()?;
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

/* ================= VALIDATION ================= */

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

/* ================= OTHER ================= */

fn uninstall(agent: String, skill: String) -> Result<()> {
    let path = agent_dir(&agent).join(skill);

    if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn list_global() -> Result<()> {
    let dir = global_dir();
    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            println!("{}", entry?.file_name().to_string_lossy());
        }
    }
    Ok(())
}

fn list_agents() -> Result<()> {
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
