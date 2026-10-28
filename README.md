mqi - Rust API's for IBM MQ
===

[![Latest version](https://img.shields.io/crates/v/mqi.svg)](https://crates.io/crates/mqi)
[![Documentation](https://docs.rs/mqi/badge.svg)](https://docs.rs/mqi)
![License](https://img.shields.io/crates/l/mqi.svg)
[![codecov](https://codecov.io/gh/advantic-au/mqi/graph/badge.svg?token=KP8423VRNZ)](https://codecov.io/gh/advantic-au/mqi)
[![Continuous Integration](https://github.com/advantic-au/mqi/actions/workflows/ci.yml/badge.svg)](https://github.com/advantic-au/mqi/actions/workflows/ci.yml?query=branch%3Adevelop+event%3Apush)

Introduction
------------

The `mqi` crate provides idiomatic Rust APIs for the IBM® MQ Interface (MQI) and MQ Administration Interface (MQAI). IBM MQ is an enterprise-grade message queuing system that enables reliable, secure, and asynchronous data exchange between applications, systems, and services.

This crate enables Rust developers to interact with IBM MQ servers, leveraging the powerful and stable IBM MQ libraries.

New to IBM MQ? Start with IBM's [overview of IBM MQ](https://www.ibm.com/products/mq).

Key Features
------------

- **MQI Integration**: Connect to IBM MQ servers to send and receive messages synchronously using robust MQI functions.
- **MQAI Administration**: Administer IBM MQ servers through the comprehensive MQAI functions.
- **Cross Platform**: Supports Windows, MacOS, and Linux.
- **Stability**: Built on the [libmqm-sys](https://crates.io/crates/libmqm-sys) crate, ensuring reliable connectivity to MQ queue managers using the IBM-supplied libraries.

Quick start
-----------

To build and run an `mqi` application, you need an IBM MQ client installation and
access to a queue manager. The client can be downloaded from IBM:

- Windows/Linux x86-64 redistributable:
  <https://public.dhe.ibm.com/ibmdl/export/pub/software/websphere/messaging/mqdev/redist/>
- macOS (x86-64/ARM64) toolkit:
  <https://public.dhe.ibm.com/ibmdl/export/pub/software/websphere/messaging/mqdev/mactoolkit/>
- Linux (x86-64/ARM64/PowerPC64le/S390X) MQ Advanced Developer:
  <https://public.dhe.ibm.com/ibmdl/export/pub/software/websphere/messaging/mqadv/>

Install the client, set `MQ_HOME` to its installation directory, and make sure
the MQ client libraries can be found when your application runs. On Linux, for
example, add `$MQ_HOME/lib64` to `LD_LIBRARY_PATH`.

For local development, start the IBM MQ container with its default developer
configuration. This creates queue manager `QM1`, the `DEV.QUEUE.1` sample queue,
and the `DEV.APP.SVRCONN` client channel used below:

```sh
docker run \
  --env LICENSE=accept \
  --env MQ_QMGR_NAME=QM1 \
  --publish 1414:1414 \
  --detach \
  --rm \
  --name QM1 \
  icr.io/ibm-messaging/mq:latest

export MQSERVER='DEV.APP.SVRCONN/TCP/localhost(1414)'
```

Accept the IBM MQ license before running the container. See the
[container usage guide](https://github.com/ibm-messaging/mq-container/blob/master/docs/usage.md)
and its [default developer configuration](https://github.com/ibm-messaging/mq-container/blob/master/docs/developer-config.md)
for details. This example uses the container's local development configuration;
configure appropriate authentication and transport security before connecting
to a queue manager outside local development.

Add `mqi` to your Rust project:

```sh
cargo add mqi
```

The following example connects using the `MQSERVER` setting above, puts a string
on `DEV.QUEUE.1`, then reads a message from that queue. For a secured channel,
configure connection credentials with `connection::Credentials`; provide real
credentials through a secret manager or environment, not source code.

```rust
use std::error::Error;

use mqi::{
    Object, constants,
    connection::NO_CONNECT_OPTIONS,
    get::GetWait,
    prelude::*,
    types::QueueName,
};

fn main() -> Result<(), Box<dyn Error>> {
    let queue_manager = mqi::connect_default(NO_CONNECT_OPTIONS).warn_as_error()?;
    {
        let queue = Object::open(
            &queue_manager,
            &(QueueName(mqstr!("DEV.QUEUE.1")), constants::MQOO_INPUT_AS_Q_DEF | constants::MQOO_OUTPUT),
        )
        .warn_as_error()?;

        queue.put_message(&(), "Hello from Rust").warn_as_error()?;

        let mut buffer = vec![0u8; 1024];
        if let Some(message) = queue
            .get_string(&GetWait::Wait(5_000), &mut *buffer)
            .warn_as_error()?
        {
            let message: std::borrow::Cow<str> = message.try_into()?;
            println!("received: {message}");
        } else {
            println!("No message arrived before the wait expired.");
        }
    }
    queue_manager.disconnect().warn_as_error()?;

    Ok(())
}
```

The example treats MQ warnings as errors. See
[handling results and warnings](./docs/recipes.md#handling-mq-warnings) for
other choices. To learn which option to use for a common MQ task, see the
[MQ task-to-option guide](./docs/recipes.md#choose-options-by-mq-task), then
browse the [MQI recipes](./docs/recipes.md) and [runnable examples](./examples/README.md).

Goals
-----

- Expose an ergonomic API over the IBM MQI libraries.
- Become the preferred API for developing MQ applications where performance and safety
  is the primary concern.
- Provide a simple and comprehensive layer over IBM MQ to connect, send and receive MQ messages,
  whilst still allowing developers to tweak the advanced options that the MQI
  library provides.
- Utilise Rust features such as lifetimes, safety guarantees, strong type system and
  invariants for a robust API.

Feature flags
-------------

| Feature        | Default | Description |
|----------------|---------|-------------|
| link           | ✔ | Support linking the MQ library at compile-time |
| dlopen2        |   | Support loading the MQ library at run-time using [`dlopen2`](https://crates.io/crates/dlopen2) |
| tracing        |   | Add tracing to the MQI and MQAI calls using the tracing crate |
| mqai           |   | Expose functions and structures related to MQAI |
| exits          |   | Expose functions and structures related to MQ exits |
| mqc_*          | mqc_9_3_0_0 | Enable features of a specific MQI library version eg `mqc_9_4_1_0` |

Version Compatibility
---------------------

- The current MSRV is 1.95. No MSRV policy has been established.
- MQ client API features are available from 9.2.0.0 through 10.0.1.0. The default
  `mqc_9_3_0_0` feature selects the minimum MQ client API version; enable a higher
  `mqc_*` feature to use newer APIs. The installed MQ client must support the
  selected feature version. IBM's support lifecycle for MQ clients is separate
  from this crate's feature range.

Status
------

This library is under heavy development. The velocity of change to the API is high and is likely to evolve. See the [progress page](PROGRESS.md) for the progress on the list of MQI functions exposed.

Support
-------

There are no guarantees of compatibility with any future versions of the crate; the API
is subject to change based on feedback and enhancements. Relentless refactoring may occur
before a stable crate is released.

This crate is provided as-is with no guarantees of support or updates.

**This crate is not approved, endorsed, acknowledged, or supported by IBM. You cannot use
IBM formal support channels (Cases/PMRs) for assistance on the use of this crate.**

Contributions
-------------

All feedback, suggestions, contributions and enhancements are welcome.

License
-------

Licensed under

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
