use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};
use tracing::{debug, info, instrument};

use crate::error::DbError;

static MIGRATOR: Migrator = sqlx::migrate!();

#[instrument(fields(db_url = %db_url.as_ref()))]
pub async fn init(db_url: impl AsRef<str>) -> Result<SqlitePool, DbError> {
    debug!("Initializing SQLite database");
    let db_url = db_url.as_ref();

    create_parent_dir_if_necessary(db_url).await?;

    let options = SqliteConnectOptions::from_str(db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    info!("Database connected, running core migrations...");

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

async fn create_parent_dir_if_necessary(db_url: impl AsRef<str>) -> Result<(), DbError> {
    let db_url = db_url.as_ref();

    if !is_url(db_url) {
        // NOTE(pencelheimer): from_str for PathBuf is infallible
        let path = PathBuf::from_str(db_url).unwrap();

        if let Some(parent_dir) = path.parent()
            && !parent_dir.as_os_str().is_empty()
            && !parent_dir.exists()
        {
            debug!("Creating database directory at {:?}", parent_dir);
            tokio::fs::create_dir_all(parent_dir).await?;
        }
    }

    Ok(())
}

fn is_url(db_url: impl AsRef<str>) -> bool {
    let db_url = db_url.as_ref();

    db_url.starts_with("sqlite:") || db_url.starts_with("file:") || db_url == ":memory:"
}
