use std::str::FromStr;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::info;

use crate::error::KoiError;

/// Migration mode for [`connect`].
///
/// - `None` — run pending migrations
/// - `Some(None)` — skip all pending migrations (baseline a pre-existing database)
/// - `Some(Some(n))` — skip pending migrations through version `n` inclusive
pub type SkipMigrations = Option<Option<i64>>;

pub async fn connect(database_url: &str, skip: SkipMigrations) -> Result<SqlitePool, KoiError> {
    ensure_database_parent_dir(database_url)?;

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|error| KoiError::Internal(format!("invalid database url: {error}")))?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    apply_migrations(&pool, skip).await?;

    Ok(pool)
}

async fn apply_migrations(pool: &SqlitePool, skip: SkipMigrations) -> Result<(), KoiError> {
    let migrator = sqlx::migrate!();

    match skip {
        None => {
            migrator.run(pool).await?;
            info!("Database migrations applied");
        }
        Some(target) => {
            migrator.skip(pool, target).await?;
            match target {
                None => info!("Database migrations skipped (baseline)"),
                Some(version) => info!("Database migrations skipped through version {version}"),
            }
        }
    }

    Ok(())
}

fn ensure_database_parent_dir(database_url: &str) -> Result<(), KoiError> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| {
                KoiError::Internal(format!("could not create database directory: {error}"))
            })?;
        }

    Ok(())
}

fn sqlite_file_path(database_url: &str) -> Option<std::path::PathBuf> {
    database_url
        .strip_prefix("sqlite://")
        .map(std::path::PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use sqlx::Row;
    use tempfile::tempdir;

    #[tokio::test]
    async fn connect_creates_database_and_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("nested").join("test.db");
        let url = format!("sqlite://{}", db_path.display());

        let pool = connect(&url, None).await.unwrap();

        assert!(db_path.is_file());

        let row = sqlx::query(
            "SELECT COUNT(*) AS count FROM sqlite_master WHERE type = 'table' AND name = 'networks'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let count: i64 = row.get("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn ensure_database_parent_dir_creates_nested_directories() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("a").join("b").join("koi.db");
        let url = format!("sqlite://{}", db_path.display());

        ensure_database_parent_dir(&url).unwrap();

        assert!(db_path.parent().unwrap().is_dir());
        assert!(!db_path.exists());

        let pool = connect(&url, None).await.unwrap();
        drop(pool);

        assert!(db_path.is_file());
    }

    #[test]
    fn sqlite_file_path_parses_relative_and_absolute_urls() {
        assert_eq!(
            sqlite_file_path("sqlite://koi.db"),
            Some(Path::new("koi.db").to_path_buf())
        );
        assert_eq!(
            sqlite_file_path("sqlite:///home/user/koi.db"),
            Some(Path::new("/home/user/koi.db").to_path_buf())
        );
    }

    #[tokio::test]
    async fn skip_baselines_existing_database() {
        let dir = tempdir().unwrap();
        let url = format!("sqlite://{}", dir.path().join("test.db").display());

        let pool = connect(&url, None).await.unwrap();
        sqlx::query("DELETE FROM _sqlx_migrations")
            .execute(&pool)
            .await
            .unwrap();
        drop(pool);

        let pool = connect(&url, Some(None)).await.unwrap();

        let row = sqlx::query("SELECT COUNT(*) AS count FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        let count: i64 = row.get("count");
        assert_eq!(count, 8);

        apply_migrations(&pool, None).await.unwrap();
    }

    #[tokio::test]
    async fn skip_respects_version_limit() {
        let dir = tempdir().unwrap();
        let url = format!("sqlite://{}", dir.path().join("test.db").display());

        let pool = connect(&url, Some(Some(5))).await.unwrap();

        let versions = sqlx::query_scalar::<_, i64>(
            "SELECT version FROM _sqlx_migrations ORDER BY version",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(versions, vec![2, 3, 4, 5]);
    }
}
