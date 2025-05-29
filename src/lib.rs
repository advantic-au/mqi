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
| `mqCreateBag`               | [`Bag::new`], [`Bag::new_lib`]                                         |
| `mqClearBag`                | [`Bag::clear`]                                                         |
| `mqDeleteBag`               | [`Bag::drop`]                                                          |
| `mqGetBag`                  | [`Object::get_bag`], [`Object::get_bag_with`]                          |
| `mqPutBag`                  | [`Object::put_bag`], [`Object::put_bag_with`]                          |
| `mqTruncateBag`             | [`Bag::truncate`]                                                      |
| `mqAddInquiry`              | [`Bag::add_inquiry`]                                                   |
| `mqDeleteItem`              | [`Bag::delete`]                                                        |
| `mqAddInteger`              | [`Bag::add`] with [`i32`]                                              |
| `mqAddIntegerFilter`        | [`Bag::add`] with [`Filter<i32>`]                                      |
| `mqAddInteger64`            | [`Bag::add`] with [`i64`]                                              |
| `mqAddString`               | [`Bag::add`] with [`EncodedString`]                                    |
| `mqAddStringFilter`         | [`Bag::add`] with [`Filter<impl EncodedString>`]                       |
| `mqAddByteString`           | [`Bag::add`] with [[`MQBYTE`](types::MQBYTE)]                          |
| `mqAddByteStringFilter`     | [`Bag::add`] with [`Filter<&[MQBYTE]>`](Filter)                        |
| `mqSetInteger`              | [`Bag::set`] with [`i32`]                                              |
| `mqSetIntegerFilter`        | [`Bag::set`] with [`Filter<i32>`]                                      |
| `mqSetInteger64`            | [`Bag::set`] with [`i64`]                                              |
| `mqAddBag`                  | [`Bag::add`] with [`Bag`]                                              |
| `mqSetString`               | [`Bag::set`] with [`EncodedString`]                                    |
| `mqSetStringFilter`         | [`Bag::set`] with [`Filter<impl EncodedString>`]                       |
| `mqSetByteString`           | [`Bag::set`] with [[`MQBYTE`](types::MQBYTE)]                          |
| `mqSetByteStringFilter`     | [`Bag::set`] with [`Filter<&[MQBYTE]>`](Filter)                        |
| `mqInquireInteger`          | [`Bag::inquire`] with [`MQLONG`](types::MQLONG)                        |
| `mqInquireIntegerFilter`    | [`Bag::inquire`] with [`Filter<MQLONG>`]                               |
| `mqInquireInteger64`        | [`Bag::inquire`] with [`MQINT64`](types::MQINT64)                      |
| `mqInquireByteString`       | [`Bag::inquire`] with [`Vec<MQBYTE>`]                                  |
| `mqInquireString`           | [`Bag::inquire`] with [`StringCcsidOwned`](StringCcsid)                |
| `mqInquireStringFilter`     | [`Bag::inquire`] with [`Filter<StringCcsidOwned>`]                     |
| `mqInquireByteStringFilter` | [`Bag::inquire`] with [`Filter<Vec<MQBYTE>>`]                          |
| `mqInquireBag`              | [`Bag::inquire`] with [`Bag`]                                          |
| `mqCountItems`              | [`Bag::count`]                                                         |
| `mqExecute`                 | [`Conn::execute`](QueueManagerAdmin::execute)                          |
| `mqBagToBuffer`             | [`Bag::to_buffer`], [`Bag::buffer_len`]                                |
| `mqBufferToBag`             | [`Bag::from_buffer`]                                                   |
| `mqInquireItemInfo`         | [`Bag::inquire`] with ([`Selector`](types::Selector), [`MQITEM`](types::MQITEM)) tuple |
| `mqTrim`                    | *Not Used*                                                             |
| `mqPad`                     | *Not Used*                                                             |

| Exits API     | Crate function(s)               |
|---------------|---------------------------------|
| `MQXCNVC`     | [`StringCcsid::try_mq_convert`] |

*/

mod mq_types;

mod mq;
pub use mq::*;

pub mod types {
    pub use lib::{MQBYTE, MQCHAR, MQINT64, MQLONG};
    pub use libmqm_constants::types::*;
    use libmqm_sys::lib;

    pub use super::mq_types::*;
}

mod struct_attach;
pub mod structs;

pub use libmqm_constants::constants;
pub mod prelude;

#[doc(hidden)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;

mod library;
pub use library::*;

mod verbs;
pub use verbs::*;

mod ccsid;
pub use ccsid::*;

mod mqstr;
pub use mqstr::*;

mod strings;
pub use strings::*;

mod encoding;
pub use encoding::*;

mod result;
pub use result::*;

mod handles;
pub use handles::*;

mod traits;
pub use traits::*;
#[cfg(feature = "mqai")]
mod mqai {
    mod filter;
    pub use filter::*;

    mod handles;
    pub use handles::*;

    mod bag;
    pub use bag::*;

    mod bag_item;
    pub use bag_item::*;

    mod execute;
    pub use execute::*;

    mod iterator;
    mod verbs;
}
#[cfg(feature = "mqai")]
pub use mqai::*;

mod support {
    pub mod conversion;
    pub mod macros;
}
pub(crate) use support::{conversion, macros};

pub mod headers;
