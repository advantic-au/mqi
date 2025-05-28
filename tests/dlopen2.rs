#![cfg(feature = "dlopen2")]

use std::{error::Error, rc::Rc};

use ::dlopen2::wrapper::Container;
use libmqm_sys::dlopen2::LoadMqm;
use mqi::MqFunctions;
use mqi::mqstr;

#[test]
fn dlopen() -> Result<(), Box<dyn Error>> {
    let lib = Rc::new(unsafe { Container::load_mqm_default()? });
    let fns = MqFunctions(lib);
    let connect_result = fns.mqconn(&mqstr!("NONEXIST"));
    assert!(connect_result.is_err());

    Ok(())
}
