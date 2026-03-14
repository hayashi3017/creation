use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::path::{Path, PathBuf};
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
    /// Run a convention-based test alias (e.g. diagram -> adapter + driver suites)
    TestScope {
        /// Scope names such as diagram, entity, auth, xtask, package:creation-service, or full
        #[arg(required = true)]
        scopes: Vec<String>,
        /// Extra args passed after `--` (e.g. -- --nocapture)
        #[arg(last = true)]
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
        Commands::TestScope { scopes, args } => {
            run_test_scopes(&scopes, &args).context("Test scope failed")?;
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestSelection {
    package: Option<String>,
    test: Option<String>,
}

impl TestSelection {
    fn full() -> Self {
        Self {
            package: None,
            test: None,
        }
    }

    fn package(package: impl Into<String>) -> Self {
        Self {
            package: Some(package.into()),
            test: None,
        }
    }

    fn integration(package: impl Into<String>, test: impl Into<String>) -> Self {
        Self {
            package: Some(package.into()),
            test: Some(test.into()),
        }
    }

    fn label(&self) -> String {
        match (&self.package, &self.test) {
            (None, None) => "full workspace".to_string(),
            (Some(package), None) => format!("package `{package}`"),
            (Some(package), Some(test)) => format!("package `{package}` test `{test}`"),
            (None, Some(test)) => format!("test `{test}`"),
        }
    }
}

fn run_test_scopes(scopes: &[String], args: &[String]) -> Result<()> {
    let selections = resolve_test_scopes(scopes)?;

    for selection in selections {
        run_tests(
            selection.package.as_deref(),
            selection.test.as_deref(),
            args,
        )
        .with_context(|| format!("Failed while running {}", selection.label()))?;
    }

    Ok(())
}

fn resolve_test_scopes(scopes: &[String]) -> Result<Vec<TestSelection>> {
    if scopes.is_empty() {
        anyhow::bail!("At least one test scope must be provided");
    }

    if scopes.iter().any(|scope| scope == "full") {
        return Ok(vec![TestSelection::full()]);
    }

    let mut selections = Vec::new();

    for scope in scopes {
        for selection in resolve_single_test_scope(scope)? {
            if !selections.contains(&selection) {
                selections.push(selection);
            }
        }
    }

    Ok(selections)
}

fn resolve_single_test_scope(scope: &str) -> Result<Vec<TestSelection>> {
    if scope == "xtask" {
        return Ok(vec![TestSelection::package("xtask")]);
    }

    if let Some(package) = scope.strip_prefix("package:") {
        return Ok(vec![TestSelection::package(package)]);
    }

    if let Some(test) = scope.strip_prefix("driver:") {
        return Ok(vec![TestSelection::integration("creation-driver", test)]);
    }

    if let Some(test) = scope.strip_prefix("adapter:") {
        return Ok(vec![TestSelection::integration("creation-adapter", test)]);
    }

    let mut selections = Vec::new();

    if test_file_exists("creation-adapter/tests", &format!("{scope}_repository.rs")) {
        selections.push(TestSelection::integration(
            "creation-adapter",
            format!("{scope}_repository"),
        ));
    }

    if test_file_exists("creation-driver/tests", &format!("{scope}.rs")) {
        selections.push(TestSelection::integration("creation-driver", scope));
    }

    if selections.is_empty() {
        anyhow::bail!(
            "Unknown test scope `{scope}`. Use full, xtask, package:<crate>, driver:<test>, adapter:<test>, or add convention-based test files."
        );
    }

    Ok(selections)
}

fn ensure_env_var(name: &str) -> Result<()> {
    std::env::var(name)
        .with_context(|| format!("{name} must be set"))
        .map(|_| ())
}

fn test_file_exists(dir: &str, file: &str) -> bool {
    repo_root().join(dir).join(file).is_file()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .to_path_buf()
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
    use super::{normalize_host_database_url, resolve_single_test_scope, resolve_test_scopes};

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

    #[test]
    fn resolve_single_test_scope_maps_convention_based_resource() {
        let selections = resolve_single_test_scope("diagram").unwrap();

        assert_eq!(selections.len(), 2);
        assert_eq!(selections[0].package.as_deref(), Some("creation-adapter"));
        assert_eq!(selections[0].test.as_deref(), Some("diagram_repository"));
        assert_eq!(selections[1].package.as_deref(), Some("creation-driver"));
        assert_eq!(selections[1].test.as_deref(), Some("diagram"));
    }

    #[test]
    fn resolve_single_test_scope_maps_driver_only_scope() {
        let selections = resolve_single_test_scope("auth").unwrap();

        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].package.as_deref(), Some("creation-driver"));
        assert_eq!(selections[0].test.as_deref(), Some("auth"));
    }

    #[test]
    fn resolve_test_scopes_supports_full_and_package_aliases() {
        let full = resolve_test_scopes(&["full".to_string()]).unwrap();
        let package = resolve_test_scopes(&["package:xtask".to_string()]).unwrap();

        assert_eq!(full.len(), 1);
        assert!(full[0].package.is_none());
        assert!(full[0].test.is_none());
        assert_eq!(package.len(), 1);
        assert_eq!(package[0].package.as_deref(), Some("xtask"));
        assert!(package[0].test.is_none());
    }
}
