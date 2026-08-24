use std::{any::Any, future::Future, pin::Pin};

use thiserror::Error;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceErrorKind {
    Unavailable,
    Serialization,
    Deadlock,
    Constraint,
    Migration,
    Internal,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("persistence operation failed ({kind:?})")]
pub struct PersistenceError {
    kind: PersistenceErrorKind,
    sqlstate: Option<String>,
}

impl PersistenceError {
    #[must_use]
    pub fn new(kind: PersistenceErrorKind, sqlstate: Option<String>) -> Self {
        Self { kind, sqlstate }
    }

    #[must_use]
    pub const fn kind(&self) -> PersistenceErrorKind {
        self.kind
    }

    #[must_use]
    pub fn sqlstate(&self) -> Option<&str> {
        self.sqlstate.as_deref()
    }

    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(
            self.kind,
            PersistenceErrorKind::Unavailable
                | PersistenceErrorKind::Serialization
                | PersistenceErrorKind::Deadlock
        )
    }
}

pub trait Transaction: Any + Send {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Error)]
pub enum UnitOfWorkError<E> {
    #[error("transaction operation failed")]
    Operation(E),
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    #[error("transaction rollback failed after operation failure: {rollback}")]
    Rollback {
        operation: E,
        rollback: PersistenceError,
    },
}

pub trait UnitOfWork: Send + Sync {
    fn execute<'a, T, E, F>(&'a self, operation: F) -> BoxFuture<'a, Result<T, UnitOfWorkError<E>>>
    where
        T: Send + 'a,
        E: Send + 'a,
        F: for<'transaction> FnOnce(
                &'transaction mut dyn Transaction,
            ) -> BoxFuture<'transaction, Result<T, E>>
            + Send
            + 'a;
}

#[cfg(test)]
mod tests {
    use super::{PersistenceError, PersistenceErrorKind};

    #[test]
    fn persistence_error_display_does_not_expose_provider_message() {
        let error =
            PersistenceError::new(PersistenceErrorKind::Constraint, Some("23505".to_owned()));

        assert_eq!(
            error.to_string(),
            "persistence operation failed (Constraint)"
        );
        assert!(!error.retryable());
        assert_eq!(error.sqlstate(), Some("23505"));
    }
}
