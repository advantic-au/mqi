use std::error::Error;

use libmqm_sys::link::LinkedMQ;
use mqi::{
    prop, sys, Completion, ConnectionOptions, Credentials, Message, MqMask, MqStr, MqStruct, MqValue, PropDetails, QueueManager, ResultCompExt
};

#[test]
fn message_handle() -> Result<(), Box<dyn Error>> {
    let (conn, ..) = QueueManager::new(
        None,
        &ConnectionOptions::default_binding().credentials(Credentials::user("app", "app")),
    )
    .warn_as_error()?;
    let handle = &conn.handle();

    let message = Message::new(&LinkedMQ, handle, MqValue::from(sys::MQCMHO_DEFAULT_VALIDATION))?;
    message
        .set_property(
            "property",
            "-1z",
            MqValue::from(sys::MQSMPO_NONE),
            &MqStruct::<sys::MQPD>::default(),
        )
        .warn_as_error()?;

    let mut name: MqStr<48> = MqStr::empty();
    let mut value: MqStr<48> = MqStr::empty();
    let bstr: &mut [u8] = &mut [0; 32];
    let mut result: sys::MQLONG = 0;
    let mut result64: sys::MQINT64 = 0;
    let mut result_detail = PropDetails::new(result);
    
    // message
    //     .inq(prop::INQUIRE_ALL, MqMask::default(), &mut name, Some(&mut result))
    //     .warn_as_error()?;
    // message
    //     .inq(prop::INQUIRE_ALL, MqMask::default(), &mut name, Some(&mut result64))
    //     .warn_as_error()?;
    // message
    //     .inq(
    //         prop::INQUIRE_ALL,
    //         MqMask::default(),
    //         &mut name,
    //         Some(&mut result_detail),
    //     )
    //     .warn_as_error()?;
    // message
    //     .inq(prop::INQUIRE_ALL, MqMask::default(), &mut name, Some(&mut value))
    //     .warn_as_error()?;
    // message
    //     .inq(prop::INQUIRE_ALL, MqMask::default(), &mut name, Some(bstr))
    //     .warn_as_error()?;

    let result: String = message.inq2(prop::INQUIRE_ALL, MqMask::default(), &mut name).warn_as_error()?;

    // let prop = message.inq_properties(prop::INQUIRE_ALL_USR).next().map(mqi::ResultCompExt::warn_as_error).transpose()?;
    // println!("{prop:?}");
    println!("{name}, {result}");

    Ok(())
}
