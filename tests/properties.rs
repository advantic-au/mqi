mod helpers;

use std::error::Error;

use helpers::mock::MockFunctions;
use mqi::{
    prelude::*,
    sys,
    values::{self, MQIMPO},
    Properties, StrCcsidOwned, ThreadNone,
};

fn mock_properties_ok(mock: &mut MockFunctions) {
    mock.expect_MQCRTMH().returning(|_, _, hmsg, comp_code, reason| {
        unsafe {
            *hmsg = 0x0a0a;
        }
        MockFunctions::mqi_outcome_ok(comp_code, reason);
    });

    mock.expect_MQDLTMH().returning(|_, _, _, comp_code, reason| {
        MockFunctions::mqi_outcome_ok(comp_code, reason);
    });
}

#[test]
fn set_property() -> Result<(), Box<dyn Error>> {
    let mut mock_library = helpers::mock::connect_ok();
    mock_properties_ok(&mut mock_library);

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
    let mut mock_library = helpers::mock::connect_ok();
    mock_properties_ok(&mut mock_library);

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
