#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

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
//!
//! ## Features
//!

#![doc = document_features::document_features!()]

//!
//! | MQI API     | Crate function(s)            |
//! |-------------|------------------------------|
//! | `MQCONN`    | *Not used*                   |
//! | `MQCONNX`   | [`connect`], [`connect_as`], [`connect_with`], [`connect_lib`], [`connect_lib_with`] |
//! | `MQPUT1`    | [`QueueManager::put_message`], [`QueueManager::put_message_with`] |
//! | `MQDISC`    | [`Connection::disconnect`], [`Connection::drop`]  |
//! | `MQOPEN`    | [`Object::open`], [`Object::open_with`] |
//! | `MQGET`     | [`Object::get_data`], [`Object::get_data_with`], [`Object::get_string`], [`Object::get_string_with`], [`Object::get_as`] |
//! | `MQPUT`     | [`Object::put_message`], [`Object::put_message_with`] |
//! | `MQINQ`     | [`Object::inq`]              |
//! | `MQSET`     | [`Object::set`]              |
//! | `MQCLOSE`   | [`Object::close`], [`Object::drop`], [`Subscription::close`], [`Subscription::drop`] |
//! | `MQSUB`     | [`Subscription::subscribe`], [`Subscription::subscribe_with`], [`Subscription::subscribe_managed`], [Subscription::subscribe_managed_with] |
//! | `MQSUBRQ`   | [`Subscription::request_retained`] |
//! | `MQCRTMH`   | [`Properties::new`]          |
//! | `MQINQMP`   | [`Properties::property`], [`Properties::property_iter`] |
//! | `MQSETMP`   | [`Properties::set_property`] |
//! | `MQDLTMP`   | [`Properties::delete_property`] |
//! | `MQBUFMH`   | [`Properties::from_buffer`], [`Properties::from_buffer_mut`] |
//! | `MQMHBUF`   | [`Properties::to_buffer`], [`Properties::to_buffer_mut`] |
//! | `MQDLTMH`   | [`Properties::close`], [`Properties::drop`]  |
//! | `MQSTAT`    | [`stat_put`], [`stat_reconnection`], [`stat_reconnection_error`] |
//! | `MQBEGIN`   | [`Syncpoint::begin`]         |
//! | `MQBACK`    | [`Syncpoint::backout`]       |
//! | `MQCMIT`    | [`Syncpoint::commit`]        |
//! | `MQCB`      | [`Connection::register_event_handler`] |
//! | `MQCTL`     | *Not implemented yet*        |
//! | `MQXCNVC`   | [`StringCcsid::try_mq_convert`] |

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

#[doc(hidden)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;
