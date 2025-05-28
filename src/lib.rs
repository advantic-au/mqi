#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

/*!
Overview
--------
Idiomatic Rust API's to the IBM® MQ Interface (MQI) and MQ Administration Interface (MQAI).

You can use `mqi` to:

- Connect to an IBM MQ server to send and receive MQ messages through the MQI functions
- Administer IBM MQ server through the MQAI functions

This crate depends and the [libmqm-sys](https://crates.io/crates/libmqm-sys) crate for
connectivity to MQ queue managers. The underlying connection uses the IBM supplied MQ libraries,
offering proven stability and performance.
*/

#![cfg_attr(
    feature = "docsrs",
    doc = r"
## Feature Flags
"
)]
#![cfg_attr(feature = "docsrs", doc = document_features::document_features!())]

/*!

| MQI function     | Crate function(s)            |
|-------------|------------------------------|
| `MQCONN`    | *Not used*                   |
| `MQCONNX`   | [`connect`], [`connect_as`], [`connect_with`], [`connect_lib`], [`connect_lib_with`] |
| `MQPUT1`    | [`QueueManager::put_message`], [`QueueManager::put_message_with`] |
| `MQDISC`    | [`Connection::disconnect`], [`Connection::drop`]  |
| `MQOPEN`    | [`Object::open`], [`Object::open_with`] |
| `MQGET`     | [`Object::get_data`], [`Object::get_data_with`], [`Object::get_string`], [`Object::get_string_with`], [`Object::get_as`] |
| `MQPUT`     | [`Object::put_message`], [`Object::put_message_with`] |
| `MQINQ`     | [`Object::inq`]              |
| `MQSET`     | [`Object::set`]              |
| `MQCLOSE`   | [`Object::close`], [`Object::drop`], [`Subscription::close`], [`Subscription::drop`] |
| `MQSUB`     | [`Subscription::subscribe`], [`Subscription::subscribe_with`], [`Subscription::subscribe_managed`], [Subscription::subscribe_managed_with] |
| `MQSUBRQ`   | [`Subscription::request_retained`] |
| `MQCRTMH`   | [`Properties::new`]          |
| `MQINQMP`   | [`Properties::property`], [`Properties::property_iter`] |
| `MQSETMP`   | [`Properties::set_property`] |
| `MQDLTMP`   | [`Properties::delete_property`] |
| `MQBUFMH`   | [`Properties::from_buffer`], [`Properties::from_buffer_mut`] |
| `MQMHBUF`   | [`Properties::to_buffer`], [`Properties::to_buffer_mut`] |
| `MQDLTMH`   | [`Properties::close`], [`Properties::drop`]  |
| `MQSTAT`    | [`stat_put`], [`stat_reconnection`], [`stat_reconnection_error`] |
| `MQBEGIN`   | [`Syncpoint::begin`]         |
| `MQBACK`    | [`Syncpoint::backout`]       |
| `MQCMIT`    | [`Syncpoint::commit`]        |
| `MQCB`      | [`Connection::register_event_handler`] |
| `MQCTL`     | *Not implemented yet*        |

| MQAI function               | Crate function(s)                                                      |
|-----------------------------|------------------------------------------------------------------------|
| `mqCreateBag`               | [`Bag::new`](`admin::Bag::new`), [`Bag::new_lib`](`admin::Bag::new_lib`) |
| `mqClearBag`                | [`Bag::clear`](`admin::Bag::clear`)                                    |
| `mqDeleteBag`               | [`Bag::drop`](`admin::Bag::drop`)                                      |
| `mqGetBag`                  | [`Object::get_bag`], [`Object::get_bag_with`]                          |
| `mqPutBag`                  | [`Object::put_bag`], [`Object::put_bag_with`]                          |
| `mqTruncateBag`             | [`Bag::truncate`](`admin::Bag::truncate`)                              |
| `mqAddInquiry`              | [`Bag::add_inquiry`](`admin::Bag::add_inquiry`)                        |
| `mqDeleteItem`              | [`Bag::delete`](`admin::Bag::delete`)                                  |
| `mqAddInteger`              | [`Bag::add`](admin::Bag::add) with [`i32`]                             |
| `mqAddIntegerFilter`        | [`Bag::add`](admin::Bag::add) with [`core::mqai::Filter<i32>`]         |
| `mqAddInteger64`            | [`Bag::add`](admin::Bag::add) with [`i64`]                             |
| `mqAddString`               | [`Bag::add`](admin::Bag::add) with [`EncodedString`]                   |
| `mqAddStringFilter`         | [`Bag::add`](admin::Bag::add) with [`core::mqai::Filter<impl EncodedString>`] |
| `mqAddByteString`           | [`Bag::add`](admin::Bag::add) with [[`types::MQBYTE`]]                 |
| `mqAddByteStringFilter`     | [`Bag::add`](admin::Bag::add) with [`Filter<&[MQBYTE]>`](core::mqai::Filter) |
| `mqSetInteger`              | [`Bag::set`](`admin::Bag::set`) with [`i32`]                           |
| `mqSetIntegerFilter`        | [`Bag::set`](`admin::Bag::set`) with [`Filter<i32>`](core::mqai::Filter) |
| `mqSetInteger64`            | [`Bag::set`](`admin::Bag::set`) with [`i64`]                           |
| `mqAddBag`                  | [`Bag::add`](admin::Bag::add) with [`admin::Bag`]                      |
| `mqSetString`               | [`Bag::set`](`admin::Bag::set`) with [`EncodedString`]                 |
| `mqSetStringFilter`         | [`Bag::set`](`admin::Bag::set`) with [`core::mqai::Filter<impl EncodedString>`] |
| `mqSetByteString`           | [`Bag::set`](`admin::Bag::set`) with [[`MQBYTE`](types::MQBYTE)]       |
| `mqSetByteStringFilter`     | [`Bag::set`](`admin::Bag::set`) with [`Filter<&[MQBYTE]>`](core::mqai::Filter) |
| `mqInquireInteger`          | [`Bag::inquire`](`admin::Bag::inquire`) with [`MQLONG`](types::MQLONG) |
| `mqInquireIntegerFilter`    | [`Bag::inquire`](`admin::Bag::inquire`) with [`Filter<MQLONG>`](core::mqai::Filter) |
| `mqInquireInteger64`        | [`Bag::inquire`](`admin::Bag::inquire`) with [`MQINT64`](types::MQINT64) |
| `mqInquireByteString`       | [`Bag::inquire`](`admin::Bag::inquire`) with [`Vec<MQBYTE>`]           |
| `mqInquireString`           | [`Bag::inquire`](`admin::Bag::inquire`) with [`StringCcsidOwned`](StringCcsid) |
| `mqInquireStringFilter`     | [`Bag::inquire`](`admin::Bag::inquire`) with [`Filter<StringCcsidOwned>`](core::mqai::Filter) |
| `mqInquireByteStringFilter` | [`Bag::inquire`](`admin::Bag::inquire`) with [`Filter<Vec<MQBYTE>>`](core::mqai::Filter) |
| `mqInquireBag`              | [`Bag::inquire`](`admin::Bag::inquire`) with [`Bag`](admin::Bag)       |
| `mqCountItems`              | [`Bag::count`](`admin::Bag::count`)                                    |
| `mqExecute`                 | [`Conn::execute`](admin::QueueManagerAdmin::execute)                   |
| `mqBagToBuffer`             | [`Bag::to_buffer`](admin::Bag::to_buffer), [`Bag::buffer_len`](admin::Bag::buffer_len) |
| `mqBufferToBag`             | [`Bag::from_buffer`](admin::Bag::from_buffer)                          |
| `mqInquireItemInfo`         | [`Bag::inquire`](admin::Bag::inquire) with ([`Selector`](types::Selector), [`MQITEM`](types::MQITEM)) tuple |
| `mqTrim`                    | *Not Used*                                                             |
| `mqPad`                     | *Not Used*                                                             |

| Exits API     | Crate function(s)               |
|---------------|---------------------------------|
| `MQXCNVC`     | [`StringCcsid::try_mq_convert`] |

*/

mod common;
mod lib_types;
mod mq;

pub mod core;

pub use common::*;
pub use mq::*;

#[cfg(feature = "mqai")]
pub mod admin;

pub mod types {
    pub use libmqm_constants::types::*;
    pub use super::mq::types::*;
    pub use super::lib_types::*;
}

pub mod structs;
mod struct_attach;

pub use libmqm_constants::constants;
pub mod prelude;

#[doc(hidden)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;
