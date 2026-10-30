# MQI recipes

These short patterns assume that `qm` is an established queue-manager connection.
For complete programs, see the linked examples, which include error handling and
command-line configuration.

## Choose options by MQ task

If you know what you want MQ to do but not which API or option to use, start
with this task-oriented map. Constants are available under `mqi::constants`;
the higher-level option types are in the modules shown below.

| I want to... | Use |
|---|---|
| Connect to a queue manager through a client channel | Set `MQSERVER`, or pass `connection::MqServer::try_from("CHANNEL/TCP/host(port)")` to `connect` |
| Supply a user ID and password when connecting | `connection::Credentials::User(user, password)` as a connection option |
| Put one message without keeping a queue open | `Connection::put_message` (MQPUT1); it opens and closes the destination as part of the call |
| Open a queue for sending | `Object::open` with `constants::MQOO_OUTPUT` |
| Open a queue for receiving | `Object::open` with `constants::MQOO_INPUT_AS_Q_DEF` |
| Receive text or arbitrary bytes | `Object::get_string` for text; `Object::get_data` for bytes |
| Wait for a message, or check immediately | `get::GetWait::Wait(milliseconds)` or `get::GetWait::NoWait` |
| Receive a particular message | `types::MessageId`, `types::CorrelationId`, or `get::MatchOptions` as get options |
| Consume and produce within one unit of work | `constants::MQGMO_SYNCPOINT` for get, `constants::MQPMO_SYNCPOINT` for put, and `Syncpoint` to commit or back out |
| Choose whether a message is persistent | `constants::MQPER_PERSISTENT`, `constants::MQPER_NOT_PERSISTENT`, or `constants::MQPER_PERSISTENCE_AS_Q_DEF` as a put option |

MQ options belong to different calls and are not interchangeable. Queue-open
options go to `Object::open`, get options to a get function, and put options to
a put function. Combine flag constants with bitwise OR only within the same
option group (for example, `MQOO_INPUT_AS_Q_DEF | MQOO_OUTPUT` when opening a
queue for both input and output). Where supported, pass multiple distinct
options as a tuple. The recipes below show these patterns in context.

## Send a message

The [quick start](../README.md#quick-start) connects and puts a string on a queue.
For an example that opens a queue or topic and reads the message body from standard
input, see [`open_put.rs`](../examples/open_put.rs).

## Receive a string

Open the queue for input, then wait up to five seconds for a message:

```rust
use mqi::{
    Object,
    constants,
    get::GetWait,
    prelude::*,
    types::QueueName,
};

let queue = Object::open(
    &qm,
    &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_INPUT_AS_Q_DEF),
)
.warn_as_error()?;

let mut buffer = vec![0u8; 1024];
if let Some(message) = queue
    .get_string(&GetWait::Wait(5_000), &mut *buffer)
    .warn_as_error()?
{
    let message: std::borrow::Cow<str> = message.try_into()?;
    println!("{message}");
}
```

`None` means no message arrived before the wait expired. Use `get_data` instead
of `get_string` when the message body is arbitrary bytes. For a publish/subscribe
consumer, see [`subscribe_managed.rs`](../examples/subscribe_managed.rs); it
creates a managed subscription to a topic, not a consumer for a regular queue.

## Request and reply

[`request_reply.rs`](../examples/request_reply.rs) shows the full pattern: open a
reply queue, put a request with its reply-to queue name, then wait for the response.
Run `cargo run --example request_reply -- --help` to see its required queue and
connection arguments.

## Inquire queue-manager details with MQAI

Enable the optional `mqai` feature (`cargo add mqi --features mqai`). The
following uses MQAI to issue an administrative inquiry and read the queue
manager name from the response bag:

```rust
use mqi::{
    bag::Bag,
    constants,
    prelude::*,
    string::StrCcsidOwned,
};

let admin = Bag::new(constants::MQCBO_ADMIN_BAG).warn_as_error()?;
let response = qm
    .execute(&admin, &constants::MQCMD_INQUIRE_Q_MGR)
    .warn_as_error()?;

if let Some(name) = response
    .inquire::<StrCcsidOwned>(constants::MQCA_Q_MGR_NAME)
    .warn_as_error()?
{
    println!("queue manager: {name:?}");
}
```

MQAI commands are processed by the queue manager's command server; the
application also needs authorization to issue the requested command. The
command's input selectors and output selectors are defined by IBM for each
MQAI command. See the [`Bag`](../src/bag/mod.rs) API and IBM's command
documentation when building other administration requests.

## Process messages transactionally

Open the source and destination queues before starting the unit of work. Use
MQ syncpoint options for both operations, and commit only after processing
succeeds:

```rust
use std::error::Error;

use mqi::{
    Object, Syncpoint, constants,
    prelude::*,
    structs,
    types::{MessageFormat, QueueName},
};

let connection = qm.connection_ref();
let input = Object::open(
    connection.clone(),
    &(QueueName(mqstr!("SOURCE.QUEUE")), constants::MQOO_INPUT_AS_Q_DEF),
)
.warn_as_error()?;
let output = Object::open(
    connection.clone(),
    &(QueueName(mqstr!("TARGET.QUEUE")), constants::MQOO_OUTPUT),
)
.warn_as_error()?;

let mut buffer = vec![0; 64 * 1024];
let syncpoint = Syncpoint::new(connection.clone());
let work = (|| -> Result<bool, Box<dyn Error>> {
    let message: Option<(&[u8], structs::MQMD)> = input
        .get_data_with(&constants::MQGMO_SYNCPOINT, &mut buffer)
        .warn_as_error()?;
    let Some((body, md)) = message else {
        return Ok(false);
    };

    output
        .put_message(
            &constants::MQPMO_SYNCPOINT,
            &(body, MessageFormat::from_mqmd(&md)),
        )
        .warn_as_error()?;
    Ok(true)
})();

match work {
    Ok(true) => {
        let _connection = syncpoint.commit().warn_as_error()?;
    }
    Ok(false) => {
        let _connection = syncpoint.backout().warn_as_error()?;
    }
    Err(error) => {
        let _connection = syncpoint.backout().warn_as_error()?;
        return Err(error);
    }
}
```

`Syncpoint::new` manages the current unit of work; it does not call `MQBEGIN`.
The MQGET and MQPUT join the unit of work because their options include
`MQGMO_SYNCPOINT` and `MQPMO_SYNCPOINT`. Use `Syncpoint::begin` only when the
unit of work must be coordinated with an external resource manager. If dropping
an unfinished `Syncpoint` triggers the backout, any backout warning or error
cannot be returned; explicit `backout` makes that outcome observable. This
minimal example propagates a backout error if both processing and backout fail,
so a production error handler may also want to record the original processing
error. For a complete forwarding program with message properties and
command-line configuration, see [`forward.rs`](../examples/forward.rs).

## Set and read message properties

Create a message handle, set a typed property, and pass the handle as a put
option. For an incoming message, pass a separate handle as the get option,
then inquire the property:

```rust
use mqi::{
    Object, Properties, constants,
    prelude::*,
    types::{MQCMHO, MQDMPO, MQIMPO, MQLONG, QueueName},
};

let queue = Object::open(
    &qm,
    &(
        QueueName(mqstr!("DEV.QUEUE.1")),
        constants::MQOO_INPUT_AS_Q_DEF | constants::MQOO_OUTPUT,
    ),
)
.warn_as_error()?;

let mut outgoing = Properties::new(&qm, MQCMHO::default())?;
let priority: MQLONG = 7;
outgoing
    .set_property("example.priority", &priority, constants::MQSMPO_SET_FIRST)
    .warn_as_error()?;
let put_options = &mut outgoing;
queue
    .put_message(&put_options, "Message with a property")
    .warn_as_error()?;

let mut incoming = Properties::new(&qm, MQCMHO::default())?;
let mut buffer = vec![0; 4096];
let get_options = &mut incoming;
if let Some(body) = queue.get_data(&get_options, &mut buffer).warn_as_error()? {
    let priority: Option<MQLONG> = incoming
        .property("example.priority", MQIMPO::default())
        .warn_as_error()?;
    println!("received {} bytes with priority {priority:?}", body.len());
}

let absent: Option<MQLONG> = incoming
    .property("optional.counter", MQIMPO::default())
    .warn_as_error()?;
assert!(absent.is_none());

incoming
    .delete_property("example.priority", MQDMPO::default())
    .warn_as_error()?;
```

`Properties` owns an MQ message handle. Use it with `MQGET`/`MQPUT` through
their option arguments as shown; the handle is deleted when dropped. Call
`Properties::close` instead when you need to observe deletion warnings or
errors. For message-handle forwarding and the `PropertyAction` options, see
[`forward.rs`](../examples/forward.rs).

Use `property_iter` with a wildcard name when the property names are not known
in advance. This example reads user properties as dynamically typed values and
returns each name alongside its value:

```rust
use mqi::{
    properties::{INQUIRE_ALL_USR, Name, Value},
    prelude::*,
    types::MQIMPO,
};

for property in incoming.property_iter::<(Value, Name<String>), _>(INQUIRE_ALL_USR, MQIMPO::default()) {
    let (value, Name(name)) = property.warn_as_error()?;
    println!("{name} = {value:?}");
}
```

The handle-based approach shown above passes a `Properties` handle to MQGET or
MQPUT. Use `Properties::to_buffer` and `Properties::from_buffer` instead when
you need to serialize properties into or restore them from message data (for
example, when the receiving application expects properties encoded in a message
header). These APIs return or accept the associated `MessageFormat`; preserve
that format with the bytes when forwarding or reconstructing the message.

## Connect with TLS

Create a TLS option using the key repository and cipher configured for your
queue manager, then pass it alongside the channel connection details:

```rust
use mqi::{
    connection::{MqServer, ThreadNone, Tls},
    mqstr,
    prelude::*,
    types::{CipherSpec, KeyRepo},
};

let key_repo = KeyRepo(mqstr!("tls.kdb"));
let cipher = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256"));
let tls = Tls::new(&key_repo, None, &cipher);
let server = MqServer::try_from("DEV.APP.SVRCONN/TCP/mq.example.com")?;

let qm = mqi::connect::<ThreadNone>(&(tls, server)).warn_as_error()?;
```

The certificate repository, cipher, channel, and host must match your MQ
configuration. See the `Tls` API documentation for certificate-label and other
TLS settings.

## Load the MQ client at runtime

Enable the `dlopen2` feature to load the MQ library at runtime instead of linking
it at build time. For example, disable default linking and keep the MQ client
version feature:

```sh
cargo add mqi --no-default-features --features dlopen2,mqc_9_3_0_0
```

[`dlopen.rs`](../examples/dlopen.rs) contains a complete example; the essential
setup is:

```rust
use dlopen2::wrapper::Container;
use mqi::prelude::*;

let mq_lib = unsafe { Container::load_mqm_default()? };
let qm = mqi::connect_lib::<mqi::connection::ThreadNone, _>(mq_lib, &())?
    .warn_as_error()?;
```

The MQ shared library must still be installed and discoverable at runtime. Loading
a shared library is unsafe; only load an MQ library from a trusted location.

## Handling MQ warnings

MQ calls can complete successfully with a warning. Use `.warn_as_error()` when a
warning should stop the operation and be returned as an error. Use
`.discard_warning()` only when the warning is intentionally unimportant to your
application. Both methods still propagate MQ errors.

The [result module](../src/result.rs) documents the result helpers and available
warning details.
