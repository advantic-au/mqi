use libmqm_default as default;

use super::option;
use crate::{
    Conn, Object, constants,
    handle::{ObjectHandle, SubscriptionHandle},
    prelude::*,
    result::{ResultComp, ResultCompErr},
    structs,
    types::{MQCO, MQLONG},
};

#[derive(Debug)]
pub struct Subscription<C: Conn> {
    handle: SubscriptionHandle,
    connection: C,
    close_options: MQCO,
}

impl<C: Conn> Subscription<C> {
    /// Close the subscription.
    ///
    /// This function uses the [`MQCLOSE`](libmqm_sys::MQCLOSE) MQ API function.
    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        s.connection
            .mq()
            .mqclose(s.connection.handle(), &mut s.handle, s.close_options)
    }

    /// Request the retained publication(s) for the subscription.
    ///
    /// This function uses the [`MQSUBRQ`](libmqm_sys::MQSUBRQ) MQ API function.
    pub fn request_retained(&self, request_options: &impl option::SubscribeRequestOption) -> ResultComp<MQLONG> {
        let mut srp = option::SubscribeRequestParam {
            sro: structs::MQSRO::new(default::MQSRO_DEFAULT),
            sr: constants::MQSR_ACTION_PUBLICATION,
        };
        request_options.apply_param(&mut srp);
        self.connection
            .mq()
            .mqsubrq(self.connection.handle(), &self.handle, srp.sr, Some(&mut srp.sro))
            .map_completion(|()| srp.sro.NumPubs)
    }
}

impl<C: Conn> Drop for Subscription<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.handle.is_closeable() {
            let _ = self
                .connection
                .mq()
                .mqclose(self.connection.handle(), &mut self.handle, self.close_options);
        }
    }
}

// Blanket implementation for SubscribeValue<C>
impl<C: Conn + Clone> Subscription<C> {
    /// This function uses the [`MQSUB`](libmqm_sys::MQSUB) MQ API function.
    pub fn subscribe<'so>(connection: C, subscribe_option: &impl option::SubscribeOption<'so>) -> ResultComp<Self> {
        Self::subscribe_as(connection, subscribe_option)
    }

    /// This function uses the [`MQSUB`](libmqm_sys::MQSUB) MQ API function.
    pub fn subscribe_with<'so, A>(connection: C, subscribe_option: &impl option::SubscribeOption<'so>) -> ResultComp<(Self, A)>
    where
        A: option::SubscribeAttr<C>,
    {
        Self::subscribe_as(connection, subscribe_option)
    }

    /// This function uses the [`MQSUB`](libmqm_sys::MQSUB) MQ API function.
    pub fn subscribe_managed_with<'so, A>(
        connection: C,
        subscribe_option: impl option::SubscribeOption<'so>,
    ) -> ResultComp<(Self, Object<C>, A)>
    where
        A: option::SubscribeAttr<C>,
    {
        Self::subscribe_as::<(Self, Option<Object<C>>, A)>(connection, &(constants::MQSO_MANAGED, subscribe_option))
            .map_completion(|(qm, queue, attr)| {
                (
                    qm,
                    queue.expect("managed queue should always be returned with MQSO_MANAGED option"),
                    attr,
                )
            })
    }

    /// This function uses the [`MQSUB`](libmqm_sys::MQSUB) MQ API function.
    pub fn subscribe_managed<'so>(
        connection: C,
        subscribe_option: impl option::SubscribeOption<'so>,
    ) -> ResultComp<(Self, Object<C>)> {
        Self::subscribe_managed_with::<()>(connection, subscribe_option).map_completion(|(sub, queue, ..)| (sub, queue))
    }

    /// This function uses the [`MQSUB`](libmqm_sys::MQSUB) MQ API function.
    pub(super) fn subscribe_as<'so, R>(
        connection: C,
        subscribe_option: &impl option::SubscribeOption<'so>,
    ) -> ResultCompErr<R, <R as option::SubscribeValue<C>>::Error>
    where
        R: option::SubscribeValue<C>,
    {
        use libmqm_sys::MQHO_NONE;

        let mut so = option::SubscribeParam {
            close_options: MQCO::default(),
            sd: structs::MQSD::new(default::MQSD_DEFAULT),
            provided_object: MQHO_NONE,
        };

        subscribe_option.apply_param(&mut so);

        R::subscribe_consume(&mut so, |param| {
            let mut obj_handle = ObjectHandle::from(param.provided_object);

            // SAFETY: Implementors of SubscribeOption must ensure the MQSD is populated correctly
            let mqsub_result = unsafe { connection.mq().mqsub(connection.handle(), &mut param.sd, &mut obj_handle) };

            mqsub_result.map_completion(|sub_handle| {
                // Create an Object if there is a unique one issued from the call
                let new_raw_handle = unsafe { obj_handle.raw_handle() };
                let object = match (param.provided_object, new_raw_handle) {
                    (_, MQHO_NONE) => None,
                    (original, new) if original == new => None,
                    (_, new) => Some(unsafe { Object::from_parts(connection.clone(), ObjectHandle::from(new)) }),
                };
                option::SubscribeState {
                    subscription: Self {
                        handle: sub_handle,
                        connection,
                        close_options: param.close_options,
                    },
                    object,
                }
            })
        })
    }
}

#[cfg(all(test, feature = "mock"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::test::mock;

    #[test]
    pub fn test_request_retained() -> Result<(), Box<dyn std::error::Error>> {
        let qm = mock::connect_ok(|mock_library| {
            mock_library
                .expect_MQSUBRQ()
                .returning(|_, _, _, sro, cc, rc| {
                    let mqsro = sro.expect("MQRSO should never be a null pointer");
                    mqsro.NumPubs = 5;
                    mock::mqi_outcome_ok(cc, rc);
                })
                .once();
            mock_library
                .expect_MQCLOSE()
                .withf(|_, &hobj, _, _, _| 1 == hobj)
                .returning(|_, _, _, cc, rc| {
                    mock::mqi_outcome_ok(cc, rc);
                })
                .once();
        });

        let sub = Subscription {
            handle: 1.into(),
            connection: qm,
            close_options: MQCO::default(),
        };

        assert_eq!(sub.request_retained(&()).warn_as_error()?, 5);

        Ok(())
    }
}
