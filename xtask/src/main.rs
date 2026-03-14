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
        /// Verify the existing .sqlx cache without updating it
        #[arg(long, default_value_t = false)]
        check: bool,
    },
    /// Run tests (the same entrypoint used by CI and local development)
    Test {
        /// Cargo package name (e.g. creation-driver)
        #[arg(long, short)]
        package: Option<String>,
        /// Integration test name (e.g. auth)
        #[arg(long)]
        test: Option<String>,
        /// Extra args passed after `--` (e.g. -- --nocapture)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
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
        Commands::SqlxPrepare { workspace, check } => {
            run_sqlx_prepare(workspace, check).context("SQLx prepare failed")?;
        }
        Commands::Test {
            package,
            test,
            args,
        } => {
            run_tests(package.as_deref(), test.as_deref(), &args).context("Test failed")?;
        }
        Commands::Docker {} => {
            run_docker().context("Migration Info failed")?;
        }
    }
    Ok(())
}

fn run_migration(source: &str) -> Result<()> {
    dotenv().ok();
    ensure_env_var("DATABASE_URL")?;

    let source_path = resolve_source_path(source);
    let mut cmd = Command::new("sqlx");
    apply_host_database_url_workaround(&mut cmd);
    let status = cmd
        .args(["database", "drop", "-y"])
        .status()
        .context("Failed to execute sqlx database drop")?;

    if status.success() {
        logger::success("database drop");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    let mut cmd = Command::new("sqlx");
    apply_host_database_url_workaround(&mut cmd);
    let status = cmd
        .args(["database", "create"])
        .status()
        .context("Failed to execute sqlx database create")?;

    if status.success() {
        logger::success("database create");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    let mut cmd = Command::new("sqlx");
    apply_host_database_url_workaround(&mut cmd);
    let status = cmd
        .args(["migrate", "run", "--source", source_path.as_str()])
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
    dotenv().ok();
    ensure_env_var("DATABASE_URL")?;

    let source_path = resolve_source_path(source);
    let mut cmd = Command::new("sqlx");
    apply_host_database_url_workaround(&mut cmd);
    let status = cmd
        .args(["migrate", "info", "--source", source_path.as_str()])
        .status()
        .context("Failed to execute sqlx info")?;

    if status.success() {
        logger::success("migration info");
    } else {
        anyhow::bail!("Migration failed with status: {}", status);
    }

    Ok(())
}

fn resolve_source_path(source: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let candidate = std::path::Path::new(source);
    if candidate.is_absolute() {
        return source.to_string();
    }

    let resolved = std::path::Path::new(root).join(source);
    resolved.to_string_lossy().to_string()
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
fn run_sqlx_prepare(workspace: bool, check: bool) -> Result<()> {
    dotenv().ok();
    ensure_env_var("DATABASE_URL")?;

    // Avoid "Invalid cross-device link (os error 18)" from incremental cache hardlinks.
    let mut cmd = Command::new("cargo");
    cmd.env("CARGO_INCREMENTAL", "0").args(["sqlx", "prepare"]);
    apply_host_database_url_workaround(&mut cmd);

    if check {
        cmd.arg("--check");
    }

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

fn run_tests(package: Option<&str>, test: Option<&str>, args: &[String]) -> Result<()> {
    dotenv().ok();

    if test_requires_database(package) {
        ensure_env_var("DATABASE_URL")?;
    }

    // Avoid "Invalid cross-device link (os error 18)" from incremental cache hardlinks.
    let mut cmd = Command::new("cargo");
    cmd.env("CARGO_INCREMENTAL", "0")
        .arg("test")
        .arg("--no-fail-fast");
    apply_host_database_url_workaround(&mut cmd);

    if let Some(package) = package {
        cmd.args(["-p", package]);
    }

    if let Some(test) = test {
        cmd.args(["--test", test]);
    }

    if !args.is_empty() {
        cmd.arg("--");
        cmd.args(args);
    }

    let status = cmd.status().context("Failed to execute cargo test")?;

    if status.success() {
        logger::success("tests");
    } else {
        anyhow::bail!("tests failed with status: {}", status);
    }

    Ok(())
}

fn ensure_env_var(name: &str) -> Result<()> {
    std::env::var(name)
        .with_context(|| format!("{name} must be set"))
        .map(|_| ())
}

fn apply_host_database_url_workaround(cmd: &mut Command) {
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        if let Some(normalized_url) = normalize_host_database_url(&database_url) {
            // Avoid sqlx::test setup DB failures like HostUnreachable/PoolTimedOut when
            // host-side commands inherit the container-only hostname `host.docker.internal`.
            cmd.env("DATABASE_URL", normalized_url);
        }
    }
}

fn normalize_host_database_url(database_url: &str) -> Option<String> {
    let scheme_end = database_url.find("://")?;
    let authority_start = scheme_end + 3;
    let authority_end = database_url[authority_start..]
        .find(['/', '?', '#'])
        .map(|index| authority_start + index)
        .unwrap_or(database_url.len());
    let authority = &database_url[authority_start..authority_end];
    let host_start = authority.rfind('@').map(|index| index + 1).unwrap_or(0);
    let host_port = &authority[host_start..];
    let host_suffix = host_port.strip_prefix("host.docker.internal")?;

    if !host_suffix.is_empty() && !host_suffix.starts_with(':') {
        return None;
    }

    let mut normalized_url = database_url.to_string();
    normalized_url.replace_range(
        authority_start + host_start..authority_end,
        &format!("localhost{host_suffix}"),
    );

    Some(normalized_url)
}

fn test_requires_database(package: Option<&str>) -> bool {
    match package {
        None => true,
        Some("creation-driver" | "creation-adapter") => true,
        Some(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_host_database_url;

    #[test]
    fn normalize_host_database_url_rewrites_container_only_hostname() {
        let normalized = normalize_host_database_url(
            "postgres://hayashi3017:password@host.docker.internal:5432/creation",
        );

        assert_eq!(
            normalized.as_deref(),
            Some("postgres://hayashi3017:password@localhost:5432/creation")
        );
    }

    #[test]
    fn normalize_host_database_url_leaves_other_hosts_untouched() {
        let normalized =
            normalize_host_database_url("postgres://hayashi3017:password@localhost:5432/creation");

        assert!(normalized.is_none());
    }
}
