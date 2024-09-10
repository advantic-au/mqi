mqi
===

[![Latest version](https://img.shields.io/crates/v/mqi.svg)](https://crates.io/crates/mqi)
[![Documentation](https://docs.rs/mqi/badge.svg)](https://docs.rs/mqi)
![License](https://img.shields.io/crates/l/mqi.svg)

Idiomatic Rust API's to the IBM® MQ Interface (MQI) and MQ Administration Interface (MQAI).

You can use `mqi` to:

- Connect to an IBM MQ server to send and receive MQ messages through the MQI functions
- Administer IBM MQ server through the MQAI functions

This crate depends and the [libmqm-sys](https://crates.io/crates/libmqm-sys) crate for
connectivity to MQ queue managers. The underlying connection uses the IBM supplied MQ libraries,
offering proven stability and performance.

Usage
-----

1. As per `libmqm-sys` crate, download and install the redistributable client from IBM:
  [https://ibm.biz/mq94redistclients]

2. Install the client in `/opt/mqm` or another location.

3. Set the MQ_HOME environment variable to the installed location.

4. Add the following to your `Cargo.toml`

    ```toml
    [dependencies]
    mqi = "0.1.0"
    ```

Example
-------

Connect to the default queue manager using the MQSERVER environment variable.

```rust
use std::error::Error;
use mqi::{
    prelude::*,
    connect_options::{ApplName, Credentials},
    mqstr,
    types::QueueName,
    QueueManager,
};

fn main() -> Result<(), Box<dyn Error>> {
    const TARGET: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    // User credentials and application name.
    // MQI will use the C API defaults of using MQSERVER environment variable
    let connect_options = (ApplName(mqstr!("readme_example")), Credentials::user("user", "password"));

    // Connect to the queue manager. Make all MQ warnings as a rust Result::Err
    let queue_manager = QueueManager::connect(connect_options).warn_as_error()?;

    // Put a single string message on the target queue with no explicit put options. Discard any warnings.
    queue_manager.put_message(TARGET, (), "Hello").discard_warning()?;

    // Queue manager disconnect - this also happens automatically on Drop.
    queue_manager.disconnect().discard_warning()?;

    Ok(())
}

```

Refer to the [examples](/examples/) folder for additional examples.

Goals
-----

- Expose an ergonomic API over the IBM MQI libraries.
- Become the preferred API for developing MQ applications where performance and safety
  is the primary concern.
- Provide a simple layer over MQ to connect, send and receive MQ messages,
  whilst still allowing developers to tweak the advanced options that the MQI
  library provides.
- Use Rust features such as lifetimes, safety guarantees, strong type system and
  invariants for a robust API.

Feature flags
-------------

| Feature        | Description |
|----------------|-------------|
| link (default) | Support linking the MQ library at compile-time |
| tracing        | Add tracing to the MQI and MQAI calls using the tracing crate |
| dlopen2        | Support loading the MQ library at run-time using [`dlopen2`](https://crates.io/crates/dlopen2) |
| mqai           | Expose the MQAI functions |
| pcf            | Generate the PCF structures |
| exits          | Generate the exit structures |

Status
------

This is under heavy development. The velocity of change to the API is high and is likely to evolve.

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
