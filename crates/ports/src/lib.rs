#![forbid(unsafe_code)]

//! Technology-neutral application ports.

pub mod transaction;

pub use transaction::{
    BoxFuture, PersistenceError, PersistenceErrorKind, Transaction, UnitOfWork, UnitOfWorkError,
};
