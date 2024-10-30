mod helpers;

use std::error::Error;

use helpers::{credentials_app, mq_library};
use mqi::{
    prelude::*,
    properties_options::{self, Attributes},
    sys, values, Properties, StrCcsidOwned, ThreadNone,
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    const PROPS: &[(&str, &str)] = &[("usr.b.x", "B"), ("usr.p.x", "A"), ("usr.c", "By"), ("usr.p.y", "C")];

    let mut mock_library = helpers::mock::MockFunctions::new();
    mock_library.connx_outcome(0x0d0d, sys::MQCC_OK, sys::MQRC_NONE);
    mock_library.disc_outcome(sys::MQCC_OK, sys::MQRC_NONE);
    let _ = mqi::connect_lib::<ThreadNone, _>(&mock_library, ()).warn_as_error()?;

    let qm = mqi::connect_lib::<ThreadNone, _>(mq_library(), credentials_app()).warn_as_error()?;

    let message = Properties::new(qm, values::MQCMHO::default())?;

    for &(name, value) in PROPS {
        message.set_property(name, value, values::MQSMPO::default()).warn_as_error()?;
    }

    for v in message.property_iter(properties_options::INQUIRE_ALL, values::MQIMPO::default()) {
        let value: (StrCcsidOwned, properties_options::Name<String>, Attributes) = v.warn_as_error()?;
        println!("{value:?}");
    }

    Ok(())
}
