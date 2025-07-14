#![cfg(all(feature = "mqai", feature = "mock", any(feature = "link", feature = "dlopen2")))]

use mqi::{
    MqStr,
    bag::Bag,
    connection::ThreadNone,
    constants,
    prelude::*,
    test,
    test::mock,
    types::{MQLONG, ObjectName},
};

#[test]
fn list_local_queues() -> Result<(), Box<dyn std::error::Error>> {
    let connection = test::mock::connect_ok(|mock_library| {
        mock::mqai::real_bag(mock_library, test::mq_library());
        mock_library.expect_mqExecute().returning(|_, _, _, _, _, _, _, cc, rc| {
            // TODO: add some return data
            test::mock::mqi_outcome_ok(cc, rc);
        });
        mock_library
            .expect_mqCountItems()
            .returning(|_, _, _, cc, rc| test::mock::mqi_outcome_ok(cc, rc));
    });

    let admin_bag = Bag::new_lib(connection.library(), constants::MQCBO_ADMIN_BAG).warn_as_error()?;
    admin_bag.add(constants::MQCA_Q_NAME.into(), "*")?.discard_warning();
    admin_bag
        .add(constants::MQIA_Q_TYPE.into(), &constants::MQQT_ALL.0)?
        .discard_warning();

    let qm = mqi::connect_lib::<ThreadNone, _>(connection.library(), &()).warn_as_error()?;
    let execute_result = qm.execute(&admin_bag, &constants::MQCMD_INQUIRE_Q).warn_as_error()?;

    for bag in execute_result.try_bag_iter(constants::MQHA_BAG_HANDLE.into())?.flatten()
    // flatten effectively ignores items that have errors
    {
        let q = bag.inquire::<ObjectName>(constants::MQCA_Q_NAME)?;
        let depth = *bag.inquire::<MQLONG>(constants::MQIA_CURRENT_Q_DEPTH)?;
        let alt_date = *bag.inquire::<MqStr<12>>(constants::MQCA_ALTERATION_DATE)?;
        let alt_time = *bag.inquire::<MqStr<12>>(constants::MQCA_ALTERATION_TIME)?;
        let ccsid = *bag.inquire::<MQLONG>(constants::MQIA_CODED_CHAR_SET_ID)?;
        let q_type = *bag.inquire::<MQLONG>(constants::MQIA_Q_TYPE)?;
        let q_pageset = *bag.inquire::<MQLONG>(constants::MQIA_PAGESET_ID)?;
        let q_desc = *bag.inquire::<MqStr<64>>(constants::MQCA_Q_DESC)?;
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
