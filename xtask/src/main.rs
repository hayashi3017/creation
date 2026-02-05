use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::process::Command;

mod logger;

/// xtask for database migration and other utilities
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run database migrations using sqlx-cli
    Migrate {
        /// Database URL (e.g. postgres://user:pass@localhost/db)
        // #[arg(long, env = "DATABASE_URL")]
        // database_url: String,
        /// Path to migration files
        #[arg(long, default_value = "../creation-adapter/migrations")]
        source: String,
    },
    MigrateInfo {
        #[arg(long, default_value = "../creation-adapter/migrations")]
        source: String,
    },
    /// Generate SQLx query cache for offline mode (loads .env)
    SqlxPrepare {
        /// Generate a workspace-level .sqlx cache
        #[arg(long, default_value_t = true)]
        workspace: bool,
    },
    Docker {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate {
            // database_url,
            source,
        } => {
            run_migration(&source).context("Migration failed")?;
        }
        Commands::MigrateInfo { source } => {
            run_migration_info(&source).context("Migration Info failed")?;
        }
        Commands::SqlxPrepare { workspace } => {
            run_sqlx_prepare(workspace).context("SQLx prepare failed")?;
        }
        Commands::Docker {} => {
            run_docker().context("Migration Info failed")?;
        }
    }
    Ok(())
}

fn run_migration(source: &str) -> Result<()> {
    let status = Command::new("sqlx")
        .args(["database", "drop", "-y"])
        .status()
        .context("Failed to execute sqlx database drop")?;

    if status.success() {
        logger::success("database drop");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    let status = Command::new("sqlx")
        .args(["database", "create"])
        .status()
        .context("Failed to execute sqlx database create")?;

    if status.success() {
        logger::success("database create");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    let status = Command::new("sqlx")
        .args(["migrate", "run", "--source", source])
        .status()
        .context("Failed to execute sqlx migrate")?;

    if status.success() {
        logger::success("migration");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    Ok(())
}

fn run_migration_info(source: &str) -> Result<()> {
    let status = Command::new("sqlx")
        .args(["migrate", "info", "--source", source])
        .status()
        .context("Failed to execute sqlx info")?;

    if status.success() {
        logger::success("migration info");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    Ok(())
}

fn run_docker() -> Result<()> {
    let root = env!("CARGO_MANIFEST_DIR");
    let env_file_path = format!("{}/../.env.docker", root);
    let status = Command::new("docker")
        .args(["compose", "--env-file", &env_file_path, "up", "-d"])
        .status()
        .context("Failed to execute docker up")?;

    if status.success() {
        logger::success("docker compose up");
    } else {
        anyhow::bail!("docker compose up failed with status: {}", status);
    }

    Ok(())
}

// FIXME: fix below error
// error: Failed to create temporary query cache directory: "/home/hayashi3017/git/creation/.sqlx"
// Error: SQLx prepare failed
fn run_sqlx_prepare(workspace: bool) -> Result<()> {
    dotenv().ok();

    // Avoid "Invalid cross-device link (os error 18)" from incremental cache hardlinks.
    let mut cmd = Command::new("cargo");
    cmd.env("CARGO_INCREMENTAL", "0")
        .env("CARGO_TARGET_DIR", "target_debug")
        .args(["sqlx", "prepare"]);

    if workspace {
        cmd.arg("--workspace");
    }

    let status = cmd
        .status()
        .context("Failed to execute cargo sqlx prepare")?;

    if status.success() {
        logger::success("sqlx prepare");
    } else {
        anyhow::bail!("sqlx prepare failed with status: {}", status);
    }

    Ok(())
}
