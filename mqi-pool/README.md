# mqi-pool

An early, synchronous [`r2d2`](https://crates.io/crates/r2d2) pool adapter for
[`mqi`](https://crates.io/crates/mqi). This crate is a prototype and is not
published as a stable API. The pool manager is generic over the mqi library
type; the example below uses linked MQ, while custom and dynamically loaded
implementations can be supplied through `mqi::connect_lib`.

Building and running requires an IBM MQ client installation as described in
the [`mqi` README](../README.md#quick-start). The default `mqi` feature links
the MQ library at build time.

## Example

```rust,no_run
use mqi::{
    connection::{Credentials, ThreadNoBlock},
    prelude::*,
    types::{ApplName, QueueName},
};
use mqi_pool::MqiConnectionManager;
use libmqm_sys::link::LinkedMq;
use r2d2::Pool;

let username = std::env::var("MQ_USER")?;
let password = std::env::var("MQ_PASSWORD")?;
let manager = MqiConnectionManager::<LinkedMq, _>::new(move || {
    let options = (
        ApplName(mqstr!("my_application")),
        Credentials::User(&username, password.as_str().into()),
    );
    mqi::connect::<ThreadNoBlock>(&options)
});
let pool = Pool::builder().max_size(8).build(manager)?;

let connection = pool.get()?;
connection
    .put_message(&QueueName(mqstr!("DEV.QUEUE.1")), &(), "Hello")
    .warn_as_error()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The factory closure should capture owned configuration and construct borrowed
connection options inside the closure. A successful MQ connection warning is
reported as a pool creation error because `r2d2` has no way to return that
warning to the borrower. If MQ reports that the connection is already owned
elsewhere, the manager rejects it without disconnecting the external owner.

## Transactions

Use the transaction guard for operations that participate in an MQ unit of
work. This does not call `MQBEGIN`; commit or back out the guard explicitly:

```rust,no_run
use mqi::{connection::ThreadNoBlock, prelude::*, types::QueueName};
use mqi_pool::MqiConnectionManager;
use libmqm_sys::link::LinkedMq;
use r2d2::Pool;

# fn run<F>(pool: &Pool<MqiConnectionManager<LinkedMq, F>>) -> Result<(), Box<dyn std::error::Error>>
# where
#     F: Fn() -> mqi::result::ResultComp<mqi_pool::MqConnection<LinkedMq>> + Send + Sync + 'static,
# {
let mut connection = pool.get()?;
let mut transaction = connection.transaction()?;

// For MQPUT/MQGET operations, pass the MQ syncpoint option as required.
transaction.put_message(
    &QueueName(mqstr!("DEV.QUEUE.1")),
    &mqi::constants::MQPMO_SYNCPOINT,
    "Transactional message",
).warn_as_error()?;

transaction.commit().warn_as_error()?;
# Ok(())
# }
```

The guard owns one `mqi::Syncpoint` for the whole transaction lifetime and
uses it for commit, backout, and drop cleanup. Dropping an unfinished guard
attempts `MQBACK`. If backout fails or returns a warning, the connection is
discarded when returned to the pool. An empty guard may receive a warning
because there is no active unit of work; prefer explicit completion when
possible. A fatal operation error does not automatically mark a connection
broken; call `mark_broken()` when its state is uncertain.

The connection remains accessible through the guard, so this prototype cannot
prevent syncpoint operations outside a transaction or detect every broken MQ
connection. It does not provide asynchronous MQ calls.
