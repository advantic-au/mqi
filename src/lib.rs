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

## Start here

- For a first connection and one-off put, start with [`connect_default`] and
  [`Connection::put_message`].
- For opening a queue, receiving messages, and repeated puts, see
  [`Object::open`] and the [MQI recipes](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md).
- For message properties, see [`Properties`] and the
  [set/read properties recipe](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md#set-and-read-message-properties).
- For transactional processing, see [`Syncpoint`] and the
  [transaction recipe](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md#process-messages-transactionally).
- For runnable programs, browse the [examples](https://github.com/advantic-au/mqi/tree/develop/examples).

The [README quick start](https://github.com/advantic-au/mqi/blob/develop/README.md#quick-start)
covers the IBM MQ client installation and connection setup required to build and run an application.

## A typical MQI workflow

The API is synchronous: each method calls the corresponding MQI operation and
returns its outcome. Connect, open an [`Object`] for the required access, perform
operations, then close the object and disconnect. This example uses the default
connection settings (including `MQSERVER`, when configured) and treats MQ
warnings as errors:

```no_run
use std::{borrow::Cow, error::Error};

use mqi::{
    Object, constants,
    connection::NO_CONNECT_OPTIONS,
    get::GetWait,
    prelude::*,
    types::QueueName,
};

#[cfg(feature = "link")]
fn main() -> Result<(), Box<dyn Error>> {
    let connection = mqi::connect_default(NO_CONNECT_OPTIONS).warn_as_error()?;
    let queue = Object::open(
        &connection,
        &(
            QueueName(mqstr!("DEV.QUEUE.1")),
            constants::MQOO_INPUT_AS_Q_DEF | constants::MQOO_OUTPUT,
        ),
    )
    .warn_as_error()?;

    queue.put_message(&(), "Hello from Rust").warn_as_error()?;

    let mut buffer = vec![0u8; 1024];
    if let Some(message) = queue
        .get_string(&GetWait::Wait(5_000), &mut *buffer)
        .warn_as_error()?
    {
        let text: Cow<str> = message.try_into()?;
        println!("received: {text}");
    }

    queue.close().warn_as_error()?;
    connection.disconnect().warn_as_error()?;
    Ok(())
}

#[cfg(not(feature = "link"))]
fn main() {}
```

### Options and MQ values

Options are passed to operations as references to values implementing that
operation's option trait. Common options are already implemented for tuples, so
you can combine (for example) a queue name and open flags as above. `Option<T>`
lets you include an option conditionally at runtime: `Some(value)` applies it,
while `None` skips it. When options set the same field, tuples are applied from
right to left, so the leftmost option takes precedence.

MQ constants are grouped into distinct newtypes such as [`MQOO`](types::MQOO),
[`MQGMO`](types::MQGMO), and [`MQPMO`](types::MQPMO). Combine flag constants
with `|` only within the option group expected by that operation; queue-open
flags belong to [`Object::open`], get flags to a get operation, and put flags
to a put operation. See the relevant module's `*Option` types for available
settings. The [MQI and MQAI function map](#mqi-and-mqai-function-map) connects
MQ verbs to their high-level methods.

### Message data and attributes

[`Object::get_data`] is suitable for arbitrary bytes. [`Object::get_string`]
is for text and returns data that you can convert to a Rust string using the
message's encoding information. Both methods take a caller-provided buffer;
size it for the largest message you intend to receive, and check the returned
`Option` because a no-wait or timed get can find no message. The get result also
provides the message format and lengths when you need to inspect or forward a
message; see [`get::GetState`] and [`get::GetValue`].

Use the `*_with` methods when you need both the operation's value and extracted
MQ attributes, or the `*_as` methods when you want to choose the returned Rust
type. The [`get`] and [`put`] modules document the supported conversions and
customization traits.

### Errors and MQ warnings

Most MQ operations return [`result::ResultComp<T>`]: an error is an MQ failure,
while a successful result is a [`result::Completion<T>`] that may carry an MQ warning. A warning
can matter even when the operation produced a value. Choose how to handle it:

- Call `.warn_as_error()?` to propagate warnings as errors.
- Call `.warning()` to inspect a warning while keeping the result.
- Call `.discard_warning()` only when the warning is intentionally irrelevant.

The [`result`] module documents the result helpers and retains the MQ verb and
reason code on [`result::Error`]. Methods ending in `close` or `disconnect` also return
completion status, so use them explicitly if cleanup warnings or errors need
to be observed; cleanup performed by `Drop` cannot return an outcome.

### Transactions and threading

For transactional message processing, pass `MQGMO_SYNCPOINT` to the get and
`MQPMO_SYNCPOINT` to the put, then commit with [`Syncpoint::commit`] or undo the
work with [`Syncpoint::backout`]. [`Syncpoint::new`] manages the current unit
of work; use [`Syncpoint::begin`] when the unit of work must be coordinated with
an external resource manager. See the
[transaction recipe](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md#process-messages-transactionally)
for an example that handles processing and backout outcomes.

The connection's threading marker controls its `Send` and `Sync` behavior and
the corresponding MQ connection-sharing option. Choose
[`connection::ThreadNone`], [`connection::ThreadNoBlock`], or
[`connection::ThreadBlock`] based on how the connection will be used;
do not share a connection across threads just by wrapping it in a synchronization
primitive.

### Optional APIs and more examples

Enable the `mqai` feature to use `bag::Bag` and `Connection::execute` for
administrative commands. MQAI commands run through the queue manager's command
server and require the appropriate MQ authorization. Enable `dlopen2` to load
the MQ client library at runtime instead of linking it at build time; see the
generated feature documentation below and the
[runtime-loading recipe](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md#load-the-mq-client-at-runtime).

For complete programs, browse the
[runnable examples](https://github.com/advantic-au/mqi/tree/develop/examples)
or the [MQI recipes](https://github.com/advantic-au/mqi/blob/develop/docs/recipes.md).
*/

#![cfg_attr(
    feature = "docsrs",
    doc = r"
## Feature Flags
"
)]
#![cfg_attr(feature = "docsrs", doc = document_features::document_features!())]

/*!

## MQI and MQAI function map

| MQI function     | Crate function(s)            |
|-------------|------------------------------|
| [`MQCONN`](libmqm_sys::MQCONN)    | *Not used*                   |
| [`MQCONNX`](libmqm_sys::MQCONNX)   | [`connect`], [`connect_default`], [`connect_as`], [`connect_with`], [`connect_lib`], [`connect_lib_with`] |
| [`MQPUT1`](libmqm_sys::MQPUT1)    | [`Connection::put_message`], [`Connection::put_message_with`] |
| [`MQDISC`](libmqm_sys::MQDISC)    | [`Connection::disconnect`], [`Connection::drop`]  |
| [`MQOPEN`](libmqm_sys::MQOPEN)    | [`Object::open`], [`Object::open_with`], [`Object::open_resolved`] |
| [`MQGET`](libmqm_sys::MQGET)     | [`Object::get_data`], [`Object::get_data_with`], [`Object::get_string`], [`Object::get_string_with`], [`Object::get_as`] |
| [`MQPUT`](libmqm_sys::MQPUT)     | [`Object::put_message`], [`Object::put_message_with`] |
| [`MQINQ`](libmqm_sys::MQINQ)     | [`Object::inquire`], [`Object::inquire_item`], [`Object::inquire_integer`] |
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
| [`MQCB`](libmqm_sys::MQCB)      | [`EventCallback::register_event_handler`], [`MessageCallback::register_message_consumer`] |
| [`MQCTL`](libmqm_sys::MQCTL)     | [`MqFunctions::mqctl`] (unsafe low-level wrapper; no managed API) |

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
| [`MQXEP`](libmqm_sys::exits::MQXEP) | [`MqFunctions::mqxep`] (unsafe low-level wrapper) |
| [`MQXCLWLN`](libmqm_sys::exits::MQXCLWLN) | *Not implemented yet* |
| [`MQXDX`](libmqm_sys::exits::MQXDX) | *Not implemented yet* |
| [`MQZEP`](libmqm_sys::exits::MQZEP) | [`MqFunctions::mqzep`] (unsafe low-level wrapper) |

*/

#![cfg_attr(docsrs, feature(doc_cfg))]

#[expect(unused_imports, reason = "Used in docsrs only")]
#[cfg(feature = "mqai")]
use bag::{Bag, Filter};

mod mq_type;

#[cfg(feature = "mqai")]
/// High-level MQAI command execution options and result types.
pub mod execute {
    pub(crate) mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

#[cfg(feature = "mqai")]
/// MQAI bags for building administrative commands and reading their results.
///
/// Enable the `mqai` feature to use this module. Bags own MQAI resources and
/// are released when dropped; explicitly delete them when the cleanup outcome
/// needs to be observed.
pub mod bag;

/// MQ scalar types and strongly typed values such as queue names, identifiers,
/// and MQ option flags.
pub mod types {
    pub use libmqm_constants::types::*;
    pub use libmqm_sys::{MQBYTE, MQCHAR, MQFLOAT32, MQFLOAT64, MQINT8, MQINT16, MQINT64, MQLONG};

    pub use super::mq_type::*;
}

mod struct_attach;
/// Lifetime-aware wrappers around IBM MQ C structures.
///
/// Use these types when an option requires direct access to an MQ structure or
/// a structure pointer. Prefer the higher-level option types where available.
pub mod structs;

/// IBM MQ constants exposed as typed values.
pub use libmqm_constants::constants;

/// Traits commonly needed to call the high-level API.
///
/// This deliberately small prelude includes helpers such as [`mqstr!`],
/// warning-handling extension methods, and connection access traits. Import
/// module-specific options from their respective modules.
pub mod prelude;

#[doc(hidden)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod test;

mod library;
pub use library::*;

/// Low-level wrappers around MQI, MQAI, and exits verbs.
///
/// Most applications should use the higher-level methods on [`Connection`],
/// [`Object`], [`Subscription`], and [`Bag`]. This module exposes the
/// underlying verb wrappers for APIs that do not yet have a managed interface.
pub mod verb {
    #[cfg(feature = "exits")]
    mod exits;
    #[cfg(feature = "mqai")]
    mod mqai;
    mod mqi;

    pub use mqi::MqInqError;
}

/// MQ character strings, CCSID-aware strings, and MQ string conversion helpers.
///
/// Use [`MqStr`] for fixed-width MQ names and [`string::EncodedString`] when an
/// API accepts text with an explicit character set.
pub mod string {
    mod ccsid;
    pub(super) mod mqstr;
    mod str_ccsid;

    pub use ccsid::CCSID;
    pub use mqstr::MqStrError;
    pub use str_ccsid::*;
}

pub use string::mqstr::{MqChar, MqStr};

/// Completion values, MQ errors, and helpers for handling MQ warnings.
pub mod result;

/// Typed wrappers around MQ connection, object, message, and (with `mqai`)
/// bag handles.
pub mod handle;

/// Traits for buffers, MQ structure access, and protected secret values.
pub mod traits;

mod support {
    pub mod conversion;
    pub mod encoding;
    pub mod macros;
    pub mod outcome;
}
pub(crate) use support::{conversion, encoding, macros};

/// MQ message-header formats and helpers for inspecting or iterating headers.
pub mod header;

#[cfg(feature = "link")]
mod link;
#[cfg(feature = "link")]
/// Functions that connect through the compile-time linked MQ client library.
pub use link::*;

/// Receive messages from an open queue or topic.
///
/// This module provides get options, result conversions, and attribute
/// extraction. Use [`Object::get_data`] for bytes or [`Object::get_string`]
/// for MQ string messages; both use a caller-provided buffer.
pub mod get {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

/// Inquire about and change attributes on an open MQ object.
///
/// Selectors and item types distinguish integer attributes from fixed-width
/// text attributes. Use [`Object::inquire`] to read one or more attributes and
/// [`Object::set`] to apply supported changes.
pub mod attribute {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

/// Connection options, threading modes, and connection-related result types.
///
/// The linked-client entry points are re-exported at the crate root; use
/// [`connect_default`] for the usual thread-confined connection or [`connect`]
/// to select a threading mode. Options such as [`connection::Credentials`],
/// [`connection::MqServer`], and [`connection::Tls`] can be combined in a
/// tuple.
pub mod connection {
    pub(super) mod function;
    pub(super) mod option;
    mod param;

    pub use function::{ConnectionEither, ThreadBlock, ThreadNoBlock, ThreadNone};
    pub use option::*;
    pub use param::*;
}
pub use connection::function::{Connection, ConnectionRef, connect_lib, connect_lib_as, connect_lib_with};

/// Options for opening an MQ queue, topic, or other object.
pub mod open {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

/// Create, inspect, modify, and serialize MQ message properties.
///
/// A [`Properties`] value owns a message handle. Pass it as a get or put option
/// to transfer properties with a message, or use the buffer methods to encode
/// properties into message data.
pub mod properties {
    pub(super) mod function;
    mod option;
    mod param;

    pub use function::MsgPropIter;
    pub use option::*;
    pub use param::*;
}
pub use properties::function::Properties;

/// Put messages to an open object.
///
/// Use [`Object::put_message`] for repeated puts to an open queue or topic;
/// use [`Connection::put_message`] for a one-off put that does not keep an
/// object open.
pub mod put {
    mod function;
    mod option;
    mod param;

    pub use option::*;
    pub use param::*;
}

/// Create and manage MQ publish/subscribe subscriptions.
///
/// Use [`Subscription::subscribe`] for a subscription to a topic and
/// [`Subscription::subscribe_managed`] when MQ should also create its
/// destination queue.
pub mod subscription {
    pub(super) mod function;
    mod option;
    mod param;

    pub use option::*;
    // pub use param::*;
}
pub use subscription::function::Subscription;

/// Options and result types for MQ connection and asynchronous put statistics.
pub mod stat {
    pub(crate) mod function;
    mod option;

    pub use option::*;
}

mod object;
/// An opened MQ object such as a queue, topic, or channel.
///
/// Objects are opened with [`Object::open`], then used for get, put, and
/// attribute operations. Dropping an object attempts to close it; call
/// [`Object::close`] when you need to handle the close completion status.
pub use object::Object;

mod syncpoint;
/// Commit or back out message operations in an MQ unit of work.
pub use syncpoint::Syncpoint;

mod event_callback;
/// Register and manage an MQ connection event callback.
pub use event_callback::EventCallback;

mod message_callback;
/// Register and manage an MQ message consumer callback on an open object.
pub use message_callback::MessageCallback;
