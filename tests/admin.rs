#![cfg(feature = "mqai")]

use mqi::admin::{selectors, Bag};
use mqi::{mqstr, prelude::*};
use mqi::{sys, Connection, ConnectionOptions, Credentials};

#[test]
fn list_local_queues() -> Result<(), Box<dyn std::error::Error>> {
    let admin_bag = Bag::new(Mask::from(sys::MQCBO_ADMIN_BAG))?;
    admin_bag.add(MqValue::from(sys::MQCA_Q_NAME), "*")?;
    admin_bag.add(MqValue::from(sys::MQIA_Q_TYPE), sys::MQQT_ALL)?;
    //admin_bag.add_inquiry(mqi::MQIA_CURRENT_Q_DEPTH)?;

    let cb = ConnectionOptions::from_mqserver("DEV.ADMIN.SVRCONN/TCP/192.168.92.15(1414)")?
        // .tls(
        //     &mqstr!("TLS_RSA_WITH_AES_128_CBC_SHA256"),
        //     Tls::new(
        //         &mqstr!("path"),
        //         Some("password"),
        //         None,
        //     ),
        // )
        .application_name(Some(mqstr!("rust_testing")))
        .credentials(Credentials::user("admin", "admin"));
    let conn = Connection::new(None, &cb).warn_as_error()?;
    let execute_result = admin_bag
        .execute(conn.handle(), MqValue::from(sys::MQCMD_INQUIRE_Q), None, None, None)
        .warn_as_error()?;

    for bag in execute_result
        .try_iter::<Bag<_, _>>(MqValue::from(sys::MQHA_BAG_HANDLE))?
        .flatten()
    // Ignore items that have errors
    {
        let q = bag.inq(&selectors::MQCA_Q_NAME, Option::None)?;
        let depth = bag.inq(&selectors::MQIA_CURRENT_Q_DEPTH, Option::None)?;
        let alt_date = bag.inq(&selectors::MQCA_ALTERATION_DATE, Option::None)?;
        let alt_time = bag.inq(&selectors::MQCA_ALTERATION_TIME, Option::None)?;
        let ccsid = bag.inq(&selectors::MQIA_CODED_CHAR_SET_ID, Option::None)?;
        let q_type = bag.inq(&selectors::MQIA_Q_TYPE, Option::None)?;
        let q_pageset = bag.inq(&selectors::MQIA_PAGESET_ID, Option::None)?;
        let q_desc = bag.inq(&selectors::MQCA_Q_DESC, Option::None)?;
        println!(
            "Queue Name: '{}'",
            String::from_utf8_lossy(q.unwrap_or_default().value())
        );
        println!("Depth: {depth:?}");
        println!("Type: {q_type:?}");
        println!("Alteration Date: '{}'", alt_date.unwrap_or_default());
        println!("Alteration Time: '{}'", alt_time.unwrap_or_default());
        println!("CCSID: {ccsid:?}");
        println!("Pageset: {q_pageset:?}");
        println!("Description: '{}'", q_desc.unwrap_or_default());
        println!("-----");
    };
    Ok(())
}
