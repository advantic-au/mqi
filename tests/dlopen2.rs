use std::{error::Error, rc::Rc};

use ::dlopen2::wrapper::Container;
use libmqm_sys::dlopen2::LoadMqm;
use mqi::{prelude::*, connect_options::Credentials, ThreadNone};

#[test]
fn dlopen() -> Result<(), Box<dyn Error>> {
    let lib = Rc::new(unsafe { Container::load_mqm_default()? });

    let _mq = mqi::connect_lib::<ThreadNone, _>(lib.clone(), Credentials::user("app", "app")).warn_as_error()?;
    let _mq2 = mqi::connect_lib::<ThreadNone, _>(lib, Credentials::user("app", "app")).warn_as_error()?;

    Ok(())
}
