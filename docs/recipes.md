# MQI recipes

These short patterns assume that `qm` is an established queue-manager connection.
For complete programs, see the linked examples, which include error handling and
command-line configuration.

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

## Process messages transactionally

Use MQ syncpoint options for both the get and put, and commit only after the work
succeeds. Back out when it fails:

```rust
use mqi::{Syncpoint, constants, prelude::*};

let connection = qm.connection_ref();
let syncpoint = Syncpoint::new(connection.clone());

// Get with constants::MQGMO_SYNCPOINT and put with constants::MQPMO_SYNCPOINT.
// If both operations succeed:
syncpoint.commit().warn_as_error()?;
```

If processing fails, call `syncpoint.backout()` instead of `commit()`. Dropping an
unfinished `Syncpoint` also backs out the unit of work. For a complete forwarding
example that handles message data, properties, and both outcomes, see
[`forward.rs`](../examples/forward.rs).

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
