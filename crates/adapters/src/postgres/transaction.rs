use adoc_ports::{
    BoxFuture, PersistenceError, PersistenceErrorKind, Transaction, UnitOfWork, UnitOfWorkError,
};
use sqlx::{PgConnection, PgPool, Postgres, pool::PoolConnection};

use super::error::map_sqlx;

#[derive(Clone)]
pub struct PgUnitOfWork {
    pool: PgPool,
}

impl PgUnitOfWork {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UnitOfWork for PgUnitOfWork {
    fn execute<'a, T, E, F>(&'a self, operation: F) -> BoxFuture<'a, Result<T, UnitOfWorkError<E>>>
    where
        T: Send + 'a,
        E: Send + 'a,
        F: for<'transaction> FnOnce(
                &'transaction mut dyn Transaction,
            ) -> BoxFuture<'transaction, Result<T, E>>
            + Send
            + 'a,
    {
        Box::pin(async move {
            let mut transaction = PgTransaction::begin(&self.pool).await?;
            match operation(&mut transaction).await {
                Ok(value) => {
                    transaction.commit().await?;
                    Ok(value)
                }
                Err(operation) => match transaction.rollback().await {
                    Ok(()) => Err(UnitOfWorkError::Operation(operation)),
                    Err(rollback) => Err(UnitOfWorkError::Rollback {
                        operation,
                        rollback,
                    }),
                },
            }
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TransactionState {
    Active,
    Completed,
}

pub(crate) struct PgTransaction {
    connection: Option<PoolConnection<Postgres>>,
    state: TransactionState,
}

impl PgTransaction {
    async fn begin(pool: &PgPool) -> Result<Self, PersistenceError> {
        let mut connection = pool.acquire().await.map_err(map_sqlx)?;
        sqlx::query("BEGIN")
            .execute(&mut *connection)
            .await
            .map_err(map_sqlx)?;
        Ok(Self {
            connection: Some(connection),
            state: TransactionState::Active,
        })
    }

    async fn commit(&mut self) -> Result<(), PersistenceError> {
        self.finish("COMMIT").await
    }

    async fn rollback(&mut self) -> Result<(), PersistenceError> {
        self.finish("ROLLBACK").await
    }

    async fn finish(&mut self, statement: &str) -> Result<(), PersistenceError> {
        let result = sqlx::query(statement)
            .execute(self.connection_mut()?)
            .await
            .map_err(map_sqlx);
        if result.is_ok() {
            self.state = TransactionState::Completed;
        }
        result.map(|_| ())
    }

    fn connection_mut(&mut self) -> Result<&mut PgConnection, PersistenceError> {
        self.connection
            .as_deref_mut()
            .filter(|_| self.state == TransactionState::Active)
            .ok_or_else(|| PersistenceError::new(PersistenceErrorKind::Internal, None))
    }
}

impl Transaction for PgTransaction {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Drop for PgTransaction {
    fn drop(&mut self) {
        if self.state == TransactionState::Active
            && let Some(connection) = self.connection.take()
        {
            drop(connection.detach());
        }
    }
}

pub(crate) fn connection(
    transaction: &mut dyn Transaction,
) -> Result<&mut PgConnection, PersistenceError> {
    transaction
        .as_any_mut()
        .downcast_mut::<PgTransaction>()
        .ok_or_else(|| PersistenceError::new(PersistenceErrorKind::Internal, None))?
        .connection_mut()
}
