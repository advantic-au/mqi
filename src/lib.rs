//! Overview
//! --------
//! Idiomatic Rust API's to the IBM® MQ Interface (MQI) and MQ Administration Interface (MQAI).
//!
//! You can use `mqi` to:
//!
//! - Connect to an IBM MQ server to send and receive MQ messages through the MQI functions
//! - Administer IBM MQ server through the MQAI functions
//!
//! This crate depends and the [libmqm-sys](https://crates.io/crates/libmqm-sys) crate for
//! connectivity to MQ queue managers. The underlying connection uses the IBM supplied MQ libraries,
//! offering proven stability and performance.
//!
mod common;
mod constants;
mod mq;

pub mod core;

pub use common::*;
pub use constants::*;
pub use mq::*;

#[cfg(feature = "mqai")]
pub mod admin;

pub mod sys {
    pub use libmqm_sys::lib::*; // Re-export mq sys library
}

pub mod prelude;
