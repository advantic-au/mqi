use std::error::Error;

use mqi::{
    prelude::*,
    connect_options::Credentials,
    properties_options::{self, Attributes},
    values, Properties, ThreadNone, StrCcsidOwned,
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    const PROPS: &[(&str, &str)] = &[("usr.b.x", "B"), ("usr.p.x", "A"), ("usr.c", "By"), ("usr.p.y", "C")];

    let conn = mqi::connect::<ThreadNone>(Credentials::user("app", "app")).warn_as_error()?;

    let message = Properties::new(conn, values::MQCMHO::default())?;

    for &(name, value) in PROPS {
        message.set_property(name, value, values::MQSMPO::default()).warn_as_error()?;
    }

    for v in message.property_iter(properties_options::INQUIRE_ALL, values::MQIMPO::default()) {
        let value: (StrCcsidOwned, properties_options::Name<String>, Attributes) = v.warn_as_error()?;
        println!("{value:?}");
    }

    Ok(())
}
