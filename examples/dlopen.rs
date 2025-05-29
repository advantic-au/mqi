use std::rc::Rc;

use anyhow::Context;
use dlopen2::wrapper::Container;
use mqi::{
    ThreadNone,
    prelude::*,
    types::{ApplName, QueueName},
};

const APP_NAME: ApplName = ApplName(mqstr!("dlopen2_example"));

fn main() -> anyhow::Result<()> {
    // Load the library. Refer to `dlopen2` for more information on loading libraries at runtime.
    // Wrap it in an Rc/Arc or use a reference if we want to use it in multiple connections.
    // Alternatively the library can be used without a reference or container for a single connection
    let mq_lib = Rc::new(unsafe { Container::load_mqm_default().context("Loading the mqm library") }?);

    // The MQ library can also be `Box::leak`ed
    // let mq_lib: &_ = Box::leak(Box::new(unsafe {
    //     Container::load_mqm_default().context("Loading the mqm library")
    // }?));

    // Connect to the queue manager using `connect_lib`` and the loaded library
    let qm = mqi::connect_lib::<ThreadNone, _>(mq_lib, &(APP_NAME))
        .warn_as_error()
        .context("Connecting to the QM")?;

    // Put a message to a queue
    qm.put_message(&QueueName(mqstr!("TEST")), &(), "test")
        .warn_as_error()
        .context("Put a message to a queue")?;

    // Disconnect
    qm.disconnect().discard_warning().context("QM disconnect")?;

    Ok(())
}
