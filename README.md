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

For local development, the IBM MQ container can provide a queue manager:

```sh
docker run --env LICENSE=accept --detach --publish 1414:1414 icr.io/ibm-messaging/mq:latest
```

Accept the IBM MQ license before running the container. See the
[container usage guide](https://github.com/ibm-messaging/mq-container/blob/master/docs/usage.md)
for configuration details.

Add `mqi` to your Rust project:

```sh
cargo add mqi
```

The following example connects using MQI's `MQSERVER` configuration, puts a
string on `DEV.QUEUE.1`, and disconnects. Set `MQSERVER` to your channel and
listener, and replace the sample credentials with credentials configured for
your queue manager.
For example, when the listener is available on localhost:

```sh
export MQSERVER='DEV.APP.SVRCONN/TCP/localhost(1414)'
```

```rust
use std::error::Error;

use mqi::{
    connection::Credentials,
    prelude::*,
    types::{ApplName, QueueName},
};

fn main() -> Result<(), Box<dyn Error>> {
    const TARGET: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    // MQI reads the client connection details from MQSERVER.
    let connect_options = (
        ApplName(mqstr!("readme_example")),
        Credentials::User("user", "password".into()),
    );

    // Connect to the queue manager. Make all MQ warnings as a rust Result::Err
    let queue_manager = mqi::connect_default(&connect_options).warn_as_error()?;

    // Put a single string message on the target queue. Discard any warnings.
    queue_manager.put_message(&TARGET, &(), "Hello").discard_warning()?;

    // Queue manager disconnect - this also happens automatically on Drop.
    queue_manager.disconnect().discard_warning()?;

    Ok(())
}
```

The example treats connection warnings as errors, but discards warnings from
put and disconnect. See [handling results and warnings](./docs/recipes.md#handling-mq-warnings)
for what those choices mean. For more tasks, see the [MQI recipes](./docs/recipes.md)
and [runnable examples](./examples/README.md).

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
| mqc_*          | mqc_9_2_0_0 | Enable features of a specific MQI library version eg `mqc_9_4_1_0` |

Version Compatibility
---------------------

- The current MSRV is 1.95. No MSRV policy has been established.
- IBM MQ client support is in line with IBM's support for MQ client. This crate supports IBM MQ client 9.2 to 9.4. Version specific features can be enabled using the `mqc_*` feature flags.

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
