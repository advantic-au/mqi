#![cfg(feature = "dlopen2")]

use std::{error::Error, rc::Rc};

use ::dlopen2::wrapper::Container;
use libmqm_sys::dlopen2::LoadMqm;
use mqi::{core::MqFunctions, sys, values};

#[test]
fn dlopen() -> Result<(), Box<dyn Error>> {
    let lib = Rc::new(unsafe { Container::load_mqm_default()? });
    let fns = MqFunctions(lib);
    let bag_result = fns.mq_create_bag(values::MQCBO(sys::MQCBO_GROUP_BAG));
    assert!(bag_result.is_ok_and(|comp| comp.warning().is_none()));

    Ok(())
}
