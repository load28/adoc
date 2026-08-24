use adoc_ports::{PersistenceError, PersistenceErrorKind};

pub(crate) fn map_sqlx(error: sqlx::Error) -> PersistenceError {
    match error {
        sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_) => PersistenceError::new(PersistenceErrorKind::Unavailable, None),
        sqlx::Error::Database(database) => {
            let sqlstate = database.code().map(|code| code.into_owned());
            let kind = match sqlstate.as_deref() {
                Some("40001") => PersistenceErrorKind::Serialization,
                Some("40P01") => PersistenceErrorKind::Deadlock,
                Some(code) if code.starts_with("23") => PersistenceErrorKind::Constraint,
                _ => PersistenceErrorKind::Internal,
            };
            PersistenceError::new(kind, sqlstate)
        }
        _ => PersistenceError::new(PersistenceErrorKind::Internal, None),
    }
}

pub(crate) fn map_migration(_error: sqlx::migrate::MigrateError) -> PersistenceError {
    PersistenceError::new(PersistenceErrorKind::Migration, None)
}
