use std::error::Error;

use mqi::{
    connect_options::Credentials,
    prelude::*,
    properties_options::{self, Attributes},
    values, Connection, Properties, ShareBlock, StrCcsidOwned,
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    const PROPS: &[(&str, &str)] = &[("usr.b.x", "B"), ("usr.p.x", "A"), ("usr.c", "By"), ("usr.p.y", "C")];

    let conn = Connection::<_, ShareBlock>::connect(Credentials::user("app", "app")).warn_as_error()?;

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
