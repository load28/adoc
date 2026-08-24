#![forbid(unsafe_code)]

//! Infrastructure adapters implementing application ports.

pub mod identity;
pub mod postgres;
pub mod rate_limit;
