use std::error::Error;

use mqi::{Object, connection::NO_CONNECT_OPTIONS, constants, get::GetWait, prelude::*, types::QueueName};

fn main() -> Result<(), Box<dyn Error>> {
    let queue_manager = mqi::connect_default(NO_CONNECT_OPTIONS).warn_as_error()?;
    {
        let queue = Object::open(
            &queue_manager,
            &(
                QueueName(mqstr!("DEV.QUEUE.1")),
                constants::MQOO_INPUT_AS_Q_DEF | constants::MQOO_OUTPUT,
            ),
        )
        .warn_as_error()?;

        queue.put_message(&(), "Hello from Rust").warn_as_error()?;

        let mut buffer = vec![0u8; 1024];
        if let Some(message) = queue.get_string(&GetWait::Wait(5_000), &mut *buffer).warn_as_error()? {
            let message: std::borrow::Cow<str> = message.try_into()?;
            println!("received: {message}");
        } else {
            println!("No message arrived before the wait expired.");
        }
    }
    queue_manager.disconnect().warn_as_error()?;

    Ok(())
}
