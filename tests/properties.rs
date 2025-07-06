#![cfg(feature = "mock")]

use std::error::Error;

use mqi::{
    Properties, constants,
    prelude::*,
    test,
    types::{MQCMHO, MQSMPO},
};
use test::mock;

#[test]
fn set_property() -> Result<(), Box<dyn Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        let mut seq = mockall::Sequence::new();
        mock::properties_ok(mock_library, 0x0d0d, 1, &mut seq);
        mock_library
            .expect_MQSETMP()
            .returning(|_, _, mqsmpo, _, _, typ, _, _, comp_code, reason| {
                assert_eq!(constants::MQTYPE_STRING, typ);
                assert_eq!(MQSMPO::default(), mqsmpo.Options);
                assert_eq!(mqsmpo.ValueCCSID, 1208);
                mock::mqi_outcome_ok(comp_code, reason);
            });
    });

    let properties = Properties::new(connection, MQCMHO::default())?;

    properties.set_property("key", "value", MQSMPO::default()).warn_as_error()?;

    Ok(())
}
