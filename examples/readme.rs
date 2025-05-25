use std::error::Error;
use mqi::{
    connect_options::Credentials,
    prelude::*,
    types::{ApplName, QueueName},
    ThreadNone,
};

fn main() -> Result<(), Box<dyn Error>> {
    const TARGET: QueueName = QueueName(mqstr!("DEV.QUEUE.1"));

    // User credentials and application name.
    // MQI will use the C API defaults, referring to MQSERVER environment variable if set.
    let connect_options = (
        ApplName(mqstr!("readme_example")),
        Credentials::User("user", "password".into()),
    );

    // Connect to the queue manager. Make all MQ warnings as a rust Result::Err
    let queue_manager = mqi::connect::<ThreadNone>(&connect_options).warn_as_error()?;

    // Put a single string message on the target queue. Discard any warnings.
    queue_manager.put_message(&TARGET, &(), "Hello").discard_warning()?;

    // Queue manager disconnect - this also happens automatically on Drop.
    queue_manager.disconnect().discard_warning()?;

    Ok(())
}
