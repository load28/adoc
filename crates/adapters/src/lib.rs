#![forbid(unsafe_code)]

//! Infrastructure adapters implementing application ports.

pub mod identity;
pub mod job_executor;
pub mod job_queue;
pub mod object_storage;
pub mod permission_cache;
pub mod postgres;
pub mod rate_limit;
pub mod search_index;
pub mod search_rebuild;
pub mod search_retrieval;
