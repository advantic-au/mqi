#![cfg(feature = "mock")]

use std::error::Error;

use test::mock::MockFunctions;
use mqi::{
    prelude::*,
    sys, test,
    values::{self, MQIMPO},
    Properties, StrCcsidOwned, ThreadNone,
};

#[test]
fn set_property() -> Result<(), Box<dyn Error>> {
    let mut mock_library = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock_library.properties_ok(0x0d0d, 1, &mut seq);

    mock_library
        .expect_MQSETMP()
        .returning(|_, _, mqsmpo, _, _, typ, _, _, comp_code, reason| {
            let mqsmpo = unsafe { *mqsmpo.cast::<sys::MQSMPO>() };

            assert_eq!(typ, sys::MQTYPE_STRING);
            assert_eq!(mqsmpo.Options, values::MQSMPO::default().value());
            assert_eq!(mqsmpo.ValueCCSID, 1208);
            MockFunctions::mqi_outcome_ok(comp_code, reason);
        });

    let qm = mqi::connect_lib::<ThreadNone, _>(&mock_library, ()).warn_as_error()?;
    let properties = Properties::new(qm, values::MQCMHO::default())?;

    properties
        .set_property("key", "value", values::MQSMPO::default())
        .warn_as_error()?;

    Ok(())
}

#[test]
fn inq_property() -> Result<(), Box<dyn Error>> {
    let mut mock_library = test::mock::connect_ok();
    let mut seq = mockall::Sequence::new();
    mock_library.properties_ok(0x0d0d, 1, &mut seq);

    mock_library
        .expect_MQINQMP()
        .returning(|_, _, _, _, _, typ, _, _, _, comp_code, reason| {
            assert_eq!(unsafe { *typ }, sys::MQTYPE_STRING);
            MockFunctions::mqi_outcome_ok(comp_code, reason);
        });

    let qm = mqi::connect_lib::<ThreadNone, _>(&mock_library, ()).warn_as_error()?;
    let properties = Properties::new(qm, values::MQCMHO::default())?;

    let _: Option<StrCcsidOwned> = properties.property("name", MQIMPO::default()).warn_as_error()?;

    Ok(())
}
