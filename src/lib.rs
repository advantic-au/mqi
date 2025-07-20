#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

/*!
Overview
--------
Idiomatic Rust API's to the IBM® MQ Interface (MQI) and MQ Administration Interface (MQAI).

You can use `mqi` to:

- Connect to an IBM MQ server to send and receive MQ messages through the MQI functions
- Administer IBM MQ server through the MQAI functions

This crate depends on the [libmqm-sys](https://crates.io/crates/libmqm-sys) crate for
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
| [`MQCONN`](libmqm_sys::MQCONN)    | *Not used*                   |
| [`MQCONNX`](libmqm_sys::MQCONNX)   | [`connect`], [`connect_as`], [`connect_with`], [`connect_lib`], [`connect_lib_with`] |
| [`MQPUT1`](libmqm_sys::MQPUT1)    | [`Connection::put_message`], [`Connection::put_message_with`] |
| [`MQDISC`](libmqm_sys::MQDISC)    | [`Connection::disconnect`], [`Connection::drop`]  |
| [`MQOPEN`](libmqm_sys::MQOPEN)    | [`Object::open`], [`Object::open_with`] |
| [`MQGET`](libmqm_sys::MQGET)     | [`Object::get_data`], [`Object::get_data_with`], [`Object::get_string`], [`Object::get_string_with`], [`Object::get_as`] |
| [`MQPUT`](libmqm_sys::MQPUT)     | [`Object::put_message`], [`Object::put_message_with`] |
| [`MQINQ`](libmqm_sys::MQINQ)     | [`Object::inquire`]              |
| [`MQSET`](libmqm_sys::MQSET)     | [`Object::set`]              |
| [`MQCLOSE`](libmqm_sys::MQCLOSE)   | [`Object::close`], [`Object::drop`], [`Subscription::close`], [`Subscription::drop`] |
| [`MQSUB`](libmqm_sys::MQSUB)     | [`Subscription::subscribe`], [`Subscription::subscribe_with`], [`Subscription::subscribe_managed`], [`Subscription::subscribe_managed_with`] |
| [`MQSUBRQ`](libmqm_sys::MQSUBRQ)   | [`Subscription::request_retained`] |
| [`MQCRTMH`](libmqm_sys::MQCRTMH)   | [`Properties::new`]          |
| [`MQINQMP`](libmqm_sys::MQINQMP)   | [`Properties::property`], [`Properties::property_iter`] |
| [`MQSETMP`](libmqm_sys::MQSETMP)   | [`Properties::set_property`] |
| [`MQDLTMP`](libmqm_sys::MQDLTMP)   | [`Properties::delete_property`] |
| [`MQBUFMH`](libmqm_sys::MQBUFMH)   | [`Properties::from_buffer`], [`Properties::from_buffer_mut`] |
| [`MQMHBUF`](libmqm_sys::MQMHBUF)   | [`Properties::to_buffer`], [`Properties::to_buffer_mut`] |
| [`MQDLTMH`](libmqm_sys::MQDLTMH)   | [`Properties::close`], [`Properties::drop`]  |
| [`MQSTAT`](libmqm_sys::MQSTAT)    | [`Connection::stat_put`], [`Connection::stat_reconnection`], [`Connection::stat_reconnection_error`] |
| [`MQBEGIN`](libmqm_sys::MQBEGIN)   | [`Syncpoint::begin`]         |
| [`MQBACK`](libmqm_sys::MQBACK)    | [`Syncpoint::backout`]       |
| [`MQCMIT`](libmqm_sys::MQCMIT)    | [`Syncpoint::commit`]        |
| [`MQCB`](libmqm_sys::MQCB)      | [`EventCallback::register_event_handler`] |
| [`MQCTL`](libmqm_sys::MQCTL)     | *Not implemented yet*        |

| MQAI function               | Crate function(s)                                                      |
|-----------------------------|------------------------------------------------------------------------|
| [`mqCreateBag`](libmqm_sys::mqai::mqCreateBag)               | [`Bag::new`], [`Bag::new_lib`]                                         |
| [`mqClearBag`](libmqm_sys::mqai::mqClearBag)                | [`Bag::clear`]                                                         |
| [`mqDeleteBag`](libmqm_sys::mqai::mqDeleteBag)               | [`Bag::drop`]                                                          |
| [`mqGetBag`](libmqm_sys::mqai::mqGetBag)                  | [`Object::get_bag`], [`Object::get_bag_with`]                          |
| [`mqPutBag`](libmqm_sys::mqai::mqPutBag)                  | [`Object::put_bag`], [`Object::put_bag_with`]                          |
| [`mqTruncateBag`](libmqm_sys::mqai::mqTruncateBag)             | [`Bag::truncate`]                                                      |
| [`mqAddInquiry`](libmqm_sys::mqai::mqAddInquiry)              | [`Bag::add_inquiry`]                                                   |
| [`mqDeleteItem`](libmqm_sys::mqai::mqDeleteItem)              | [`Bag::delete`]                                                        |
| [`mqAddInteger`](libmqm_sys::mqai::mqAddInteger)              | [`Bag::add`] with [`MQLONG`](types::MQLONG)                            |
| [`mqAddIntegerFilter`](libmqm_sys::mqai::mqAddIntegerFilter)        | [`Bag::add`] with [`Filter<MQLONG>`]                            |
| [`mqAddInteger64`](libmqm_sys::mqai::mqAddInteger64)            | [`Bag::add`] with [`MQINT64`](types::MQINT64)                          |
| [`mqAddString`](libmqm_sys::mqai::mqAddString)               | [`Bag::add`] with [`EncodedString`](string::EncodedString)                                    |
| [`mqAddStringFilter`](libmqm_sys::mqai::mqAddStringFilter)         | [`Bag::add`] with [`Filter<impl EncodedString>`]                       |
| [`mqAddByteString`](libmqm_sys::mqai::mqAddByteString)           | [`Bag::add`] with [[`MQBYTE`](types::MQBYTE)]                          |
| [`mqAddByteStringFilter`](libmqm_sys::mqai::mqAddByteStringFilter)     | [`Bag::add`] with [`Filter<&[MQBYTE]>`](Filter)                        |
| [`mqSetInteger`](libmqm_sys::mqai::mqSetInteger)              | [`Bag::set`] with [`MQLONG`](types::MQLONG)                            |
| [`mqSetIntegerFilter`](libmqm_sys::mqai::mqSetIntegerFilter)        | [`Bag::set`] with [`Filter<MQLONG>`]                                      |
| [`mqSetInteger64`](libmqm_sys::mqai::mqSetInteger64)            | [`Bag::set`] with [`MQINT64`](types::MQINT64)                                              |
| [`mqAddBag`](libmqm_sys::mqai::mqAddBag)                  | [`Bag::add`] with [`Bag`]                                              |
| [`mqSetString`](libmqm_sys::mqai::mqSetString)               | [`Bag::set`] with [`EncodedString`](string::EncodedString)                                    |
| [`mqSetStringFilter`](libmqm_sys::mqai::mqSetStringFilter)         | [`Bag::set`] with [`Filter<impl EncodedString>`]                       |
| [`mqSetByteString`](libmqm_sys::mqai::mqSetByteString)           | [`Bag::set`] with [[`MQBYTE`](types::MQBYTE)]                          |
| [`mqSetByteStringFilter`](libmqm_sys::mqai::mqSetByteStringFilter)     | [`Bag::set`] with [`Filter<&[MQBYTE]>`](Filter)                        |
| [`mqInquireInteger`](libmqm_sys::mqai::mqInquireInteger)          | [`Bag::inquire`] with [`MQLONG`](types::MQLONG)                        |
| [`mqInquireIntegerFilter`](libmqm_sys::mqai::mqInquireIntegerFilter)    | [`Bag::inquire`] with [`Filter<MQLONG>`]                               |
| [`mqInquireInteger64`](libmqm_sys::mqai::mqInquireInteger64)        | [`Bag::inquire`] with [`MQINT64`](types::MQINT64)                      |
| [`mqInquireByteString`](libmqm_sys::mqai::mqInquireByteString)       | [`Bag::inquire`] with [`Vec<MQBYTE>`]                                  |
| [`mqInquireString`](libmqm_sys::mqai::mqInquireString)           | [`Bag::inquire`] with [`StringCcsidOwned`](string::StringCcsid)                |
| [`mqInquireStringFilter`](libmqm_sys::mqai::mqInquireStringFilter)     | [`Bag::inquire`] with [`Filter<StringCcsidOwned>`]                     |
| [`mqInquireByteStringFilter`](libmqm_sys::mqai::mqInquireByteStringFilter) | [`Bag::inquire`] with [`Filter<Vec<MQBYTE>>`]                          |
| [`mqInquireBag`](libmqm_sys::mqai::mqInquireBag)              | [`Bag::inquire`] with [`Bag`]                                          |
| [`mqCountItems`](libmqm_sys::mqai::mqCountItems)              | [`Bag::count`]                                                         |
| [`mqExecute`](libmqm_sys::mqai::mqExecute)                 | [`Connection::execute`]                          |
| [`mqBagToBuffer`](libmqm_sys::mqai::mqBagToBuffer)             | [`Bag::to_buffer`], [`Bag::buffer_len`]                                |
| [`mqBufferToBag`](libmqm_sys::mqai::mqBufferToBag)             | [`Bag::from_buffer`]                                                   |
| [`mqInquireItemInfo`](libmqm_sys::mqai::mqInquireItemInfo)         | [`Bag::inquire`] with ([`Selector`](types::Selector), [`MQITEM`](types::MQITEM)) tuple |
| [`mqTrim`](libmqm_sys::mqai::mqTrim)                    | *Not Used*                                                             |
| [`mqPad`](libmqm_sys::mqai::mqPad)                     | *Not Used*                                                             |

| Exits API     | Crate function(s)               |
|---------------|---------------------------------|
| [`MQXCNVC`](libmqm_sys::MQXCNVC) | [`StringCcsid::try_mq_convert`](string::StringCcsid::try_mq_convert) |
| [`MQXEP`](libmqm_sys::exits::MQXEP) | *Not implemented yet* |
| [`MQXCLWLN`](libmqm_sys::exits::MQXCLWLN) | *Not implemented yet* |
| [`MQXDX`](libmqm_sys::exits::MQXDX) | *Not implemented yet* |
| [`MQZEP`](libmqm_sys::exits::MQZEP) | *Not implemented yet* |

*/

#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg, doc_cfg_hide))]

#[expect(unused_imports, reason = "Used in docsrs only")]
#[cfg(feature = "mqai")]
use bag::{Bag, Filter};

mod mq_type;

#[cfg(feature = "mqai")]
pub mod execute {
    pub(crate) mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

#[cfg(feature = "mqai")]
pub mod bag;

pub mod types {
    pub use libmqm_constants::types::*;
    pub use libmqm_sys::{MQBYTE, MQCHAR, MQFLOAT32, MQFLOAT64, MQINT8, MQINT16, MQINT64, MQLONG};

    pub use super::mq_type::*;
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

pub mod verb {
    #[cfg(feature = "mqai")]
    mod mqai;
    mod mqi;

    pub use mqi::MqInqError;
}

pub mod string {
    mod ccsid;
    pub(super) mod mqstr;
    mod str_ccsid;

    pub use ccsid::CCSID;
    pub use mqstr::MqStrError;
    pub use str_ccsid::*;
}

pub use string::mqstr::{MqChar, MqStr};

pub mod result;

pub mod handle;

pub mod traits;

mod support {
    pub mod conversion;
    pub mod encoding;
    pub mod macros;
    pub mod outcome;
}
pub(crate) use support::{conversion, encoding, macros};

pub mod header;

#[cfg(feature = "link")]
mod link;
#[cfg(feature = "link")]
pub use link::*;

pub mod get {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

pub mod attribute {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

pub mod connection {
    pub(super) mod function;
    pub(super) mod option;
    mod param;

    pub use function::{ThreadBlock, ThreadNoBlock, ThreadNone};
    pub use option::*;
    pub use param::*;
}
pub use connection::function::{Connection, ConnectionRef, connect_lib, connect_lib_as, connect_lib_with};

pub mod open {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

pub mod properties {
    pub(super) mod function;
    mod option;
    mod param;

    pub use function::MsgPropIter;
    pub use option::*;
    pub use param::*;
}
pub use properties::function::Properties;

pub mod put {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

pub mod subscription {
    pub(super) mod function;
    mod option;
    mod param;

    pub use option::*;
    // pub use param::*;
}
pub use subscription::function::Subscription;

pub mod stat {
    pub(crate) mod function;
    mod option;

    pub use option::*;
}

mod object;
pub use object::Object;

mod syncpoint;
pub use syncpoint::Syncpoint;

mod callback;
pub use callback::EventCallback;
