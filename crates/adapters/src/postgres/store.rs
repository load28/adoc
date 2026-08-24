use std::time::Duration;

use adoc_ports::{PersistenceError, PersistenceErrorKind};
use sqlx::{PgPool, Row, migrate::Migrator, postgres::PgPoolOptions};

use super::error::{map_migration, map_sqlx};

static MIGRATOR: Migrator = sqlx::migrate!("../../infra/migrations");

#[derive(Clone, Copy)]
pub struct DatabaseSettings<'a> {
    pub url: &'a str,
    pub max_connections: u32,
    pub application_name: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresPreflight {
    pub server_major_version: u32,
    pub applied_migrations: usize,
}

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(settings: DatabaseSettings<'_>) -> Result<Self, PersistenceError> {
        validate_settings(settings)?;
        let application_name = settings.application_name.to_owned();
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(10 * 60)))
            .max_lifetime(Some(Duration::from_secs(30 * 60)))
            .after_connect(move |connection, _metadata| {
                let application_name = application_name.clone();
                Box::pin(async move {
                    sqlx::query("SET TIME ZONE 'UTC'")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("SET lock_timeout = '5s'")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("SET statement_timeout = '30s'")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("SELECT set_config('application_name', $1, false)")
                        .bind(&application_name)
                        .execute(&mut *connection)
                        .await?;
                    Ok(())
                })
            })
            .connect(settings.url)
            .await
            .map_err(map_sqlx)?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), PersistenceError> {
        MIGRATOR.run(&self.pool).await.map_err(map_migration)
    }

    pub async fn preflight(&self) -> Result<PostgresPreflight, PersistenceError> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx)?;
        let version: String = sqlx::query_scalar("SHOW server_version_num")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx)?;
        let version = version
            .parse::<u32>()
            .map_err(|_| PersistenceError::new(PersistenceErrorKind::Internal, None))?;
        let server_major_version = version / 10_000;
        if server_major_version < 16 {
            return Err(PersistenceError::new(
                PersistenceErrorKind::Unavailable,
                None,
            ));
        }

        let applied = sqlx::query("SELECT version, success FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx)?;
        let expected_versions = MIGRATOR
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>();
        let applied_versions = applied
            .iter()
            .filter(|row| row.get::<bool, _>("success"))
            .map(|row| row.get::<i64, _>("version"))
            .collect::<Vec<_>>();
        if applied_versions != expected_versions {
            return Err(PersistenceError::new(PersistenceErrorKind::Migration, None));
        }

        Ok(PostgresPreflight {
            server_major_version,
            applied_migrations: applied_versions.len(),
        })
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

fn validate_settings(settings: DatabaseSettings<'_>) -> Result<(), PersistenceError> {
    let application_name_valid = !settings.application_name.is_empty()
        && settings.application_name.len() <= 63
        && settings
            .application_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if settings.url.is_empty()
        || !(1..=200).contains(&settings.max_connections)
        || !application_name_valid
    {
        return Err(PersistenceError::new(PersistenceErrorKind::Internal, None));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use adoc_ports::PersistenceErrorKind;

    use super::{DatabaseSettings, validate_settings};

    #[test]
    fn database_settings_reject_invalid_pool_and_application_name() {
        for settings in [
            DatabaseSettings {
                url: "",
                max_connections: 1,
                application_name: "adoc-api",
            },
            DatabaseSettings {
                url: "postgres://redacted",
                max_connections: 0,
                application_name: "adoc-api",
            },
            DatabaseSettings {
                url: "postgres://redacted",
                max_connections: 1,
                application_name: "invalid name",
            },
        ] {
            assert_eq!(
                validate_settings(settings).unwrap_err().kind(),
                PersistenceErrorKind::Internal
            );
        }
    }
}
