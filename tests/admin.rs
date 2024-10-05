#![cfg(feature = "mqai")]

use mqi::{prelude::*, ThreadNone};
use mqi::admin::Bag;
use mqi::connect_options::{ApplName, Credentials, MqServer};
use mqi::values;
use mqi::types::ObjectName;
use mqi::MqStr;
use mqi::sys;

#[test]
fn list_local_queues() -> Result<(), Box<dyn std::error::Error>> {
    let admin_bag = Bag::new(values::MQCBO(sys::MQCBO_ADMIN_BAG)).warn_as_error()?;
    admin_bag.add(values::MqaiSelector(sys::MQCA_Q_NAME), "*")?.discard_warning();
    admin_bag
        .add(values::MqaiSelector(sys::MQIA_Q_TYPE), &sys::MQQT_ALL)?
        .discard_warning();

    let qm = mqi::connect::<ThreadNone>((
        ApplName(mqstr!("rust_testing")),
        MqServer::try_from("DEV.ADMIN.SVRCONN/TCP/192.168.92.15(1414)")?,
        Credentials::user("admin", "admin"),
    ))
    .warn_as_error()?;

    let execute_result = qm.execute(&admin_bag, values::MQCMD(sys::MQCMD_INQUIRE_Q)).warn_as_error()?;

    for bag in execute_result
        .try_bag_iter(values::MqaiSelector(sys::MQHA_BAG_HANDLE))?
        .flatten()
    // flatten effectively ignores items that have errors
    {
        let q = bag.inquire::<ObjectName>(sys::MQCA_Q_NAME)?;
        let depth = *bag.inquire::<sys::MQLONG>(sys::MQIA_CURRENT_Q_DEPTH)?;
        let alt_date = *bag.inquire::<MqStr<12>>(sys::MQCA_ALTERATION_DATE)?;
        let alt_time = *bag.inquire::<MqStr<12>>(sys::MQCA_ALTERATION_TIME)?;
        let ccsid = *bag.inquire::<sys::MQLONG>(sys::MQIA_CODED_CHAR_SET_ID)?;
        let q_type = *bag.inquire::<sys::MQLONG>(sys::MQIA_Q_TYPE)?;
        let q_pageset = *bag.inquire::<sys::MQLONG>(sys::MQIA_PAGESET_ID)?;
        let q_desc = *bag.inquire::<MqStr<64>>(sys::MQCA_Q_DESC)?;
        println!("Queue Name: {}", q.unwrap_or_default());
        println!("Depth: {}", depth.map_or("{n/a}".to_string(), |t| t.to_string()));
        println!("Type: {}", q_type.map_or("{n/a}".to_string(), |t| t.to_string()));
        println!("Alteration Date: '{}'", alt_date.unwrap_or_default());
        println!("Alteration Time: '{}'", alt_time.unwrap_or_default());
        println!("CCSID: {ccsid:?}");
        println!("Pageset: {q_pageset:?}");
        println!("Description: '{}'", q_desc.unwrap_or_default());
        println!("-----");
    }
    Ok(())
}
