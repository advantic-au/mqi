use crate::{
    core::{self, ObjectHandle},
    prelude::*,
    sys, values, Error, ResultComp, ResultCompErr,
};

use super::{Conn, MqStruct, Object};

use libmqm_default as default;

#[derive(Debug)]
pub struct Subscription<C: Conn> {
    handle: core::SubscriptionHandle,
    connection: C,
    close_options: values::MQCO,
}

pub struct SubscribeState<C: Conn> {
    pub subscription: Subscription<C>,
    pub object: Option<Object<C>>,
}

#[derive(Debug)]
pub struct SubscribeParam<'a> {
    pub sd: MqStruct<'a, sys::MQSD>,
    pub close_options: values::MQCO,
    pub provided_object: sys::MQLONG,
}

#[derive(Debug)]
pub struct SubscribeRequestParam {
    pub sro: MqStruct<'static, sys::MQSRO>,
    pub sr: values::MQSR,
}

impl<C: Conn> Subscription<C> {
    /// Close the subscription.
    ///
    /// This utilises the MQI function `MQCLOSE`.
    pub fn close(self) -> ResultComp<()> {
        let mut s = self;
        s.connection
            .mq()
            .mqclose(s.connection.handle(), &mut s.handle, s.close_options)
    }

    /// Request the retained publication(s) for the subscription.
    ///
    /// This utilises the MQI function `MQSUBRQ`.
    pub fn request_retained(&self, request_options: &impl SubscribeRequestOption) -> ResultComp<sys::MQLONG> {
        let mut srp = SubscribeRequestParam {
            sro: MqStruct::new(default::MQSRO_DEFAULT),
            sr: values::MQSR(sys::MQSR_ACTION_PUBLICATION),
        };
        request_options.apply_param(&mut srp);
        self.connection
            .mq()
            .mqsubrq(self.connection.handle(), &self.handle, srp.sr, &mut srp.sro)
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

pub trait SubscribeValue<C: Conn> {
    type Error: From<Error> + std::fmt::Debug;

    fn subscribe_consume<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultCompErr<Self, Self::Error>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: std::marker::Sized;
}

pub trait SubscribeAttr<C: Conn> {
    fn subscribe_extract<'so, F>(param: &mut SubscribeParam<'so>, mqi: F) -> ResultComp<(Self, SubscribeState<C>)>
    where
        F: FnOnce(&mut SubscribeParam<'so>) -> ResultComp<SubscribeState<C>>,
        Self: Sized;
}

/// A trait that manipulates the parameters to the [`mqsub`](`crate::core::MqFunctions::mqsub`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `SubscribeOption` so it can't be used as an argument for MQI subscribe"
)]
pub trait SubscribeOption<'so> {
    fn apply_param(&self, param: &mut SubscribeParam<'so>);
}

pub trait SubscribeRequestOption {
    fn apply_param(&self, param: &mut SubscribeRequestParam);
}

// Blanket implementation for SubscribeValue<C>
impl<C: Conn + Clone> Subscription<C> {
    pub fn subscribe<'so>(connection: C, subscribe_option: &impl SubscribeOption<'so>) -> ResultComp<Self> {
        Self::subscribe_as(connection, subscribe_option)
    }

    pub fn subscribe_with<'so, A>(connection: C, subscribe_option: &impl SubscribeOption<'so>) -> ResultComp<(Self, A)>
    where
        A: SubscribeAttr<C>,
    {
        Self::subscribe_as(connection, subscribe_option)
    }

    pub fn subscribe_managed_with<'so, A>(
        connection: C,
        subscribe_option: impl SubscribeOption<'so>,
    ) -> ResultComp<(Self, Object<C>, A)>
    where
        A: SubscribeAttr<C>,
    {
        Self::subscribe_as::<(Self, Option<Object<C>>, A)>(connection, &(values::MQSO(sys::MQSO_MANAGED), subscribe_option))
            .map_completion(|(qm, queue, attr)| {
                (
                    qm,
                    queue.expect("managed queue should always be returned with MQSO_MANAGED option"),
                    attr,
                )
            })
    }

    pub fn subscribe_managed<'so>(connection: C, subscribe_option: impl SubscribeOption<'so>) -> ResultComp<(Self, Object<C>)> {
        Self::subscribe_managed_with::<()>(connection, subscribe_option).map_completion(|(sub, queue, ..)| (sub, queue))
    }

    pub(super) fn subscribe_as<'so, R>(
        connection: C,
        subscribe_option: &impl SubscribeOption<'so>,
    ) -> ResultCompErr<R, <R as SubscribeValue<C>>::Error>
    where
        R: SubscribeValue<C>,
    {
        let mut so = SubscribeParam {
            close_options: values::MQCO::default(),
            sd: MqStruct::new(default::MQSD_DEFAULT),
            provided_object: sys::MQHO_NONE,
        };

        subscribe_option.apply_param(&mut so);

        R::subscribe_consume(&mut so, |param| {
            let mut obj_handle = ObjectHandle::from(param.provided_object);
            connection
                .mq()
                .mqsub(connection.handle(), &mut param.sd, &mut obj_handle)
                .map_completion(|sub_handle| {
                    // Create an Object if there is a unique one issued from the call
                    let new_raw_handle = unsafe { obj_handle.raw_handle() };
                    let object = match (param.provided_object, new_raw_handle) {
                        (_, sys::MQHO_NONE) => None,
                        (original, new) if original == new => None,
                        (_, new) => Some(unsafe { Object::from_parts(connection.clone(), ObjectHandle::from(new)) }),
                    };
                    SubscribeState {
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
    use crate::{
        prelude::*,
        sys,
        test::mock::{self, MockFunctions},
        values, MqStruct,
    };

    use super::Subscription;

    #[test]
    pub fn test_request_retained() -> Result<(), Box<dyn std::error::Error>> {
        let qm = mock::connect_ok(|mock_library| {
            mock_library
            .expect_MQSUBRQ()
            .returning(|_, _, _, sro, cc, rc| {
                let mqsro: *mut MqStruct<sys::MQSRO> = sro.cast();
                unsafe {
                    (*mqsro).NumPubs = 5;
                }
                MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .once();
        mock_library
            .expect_MQCLOSE()
            .withf(|_, &hobj, _, _, _| 1 == unsafe { *hobj })
            .returning(|_, _, _, cc, rc| {
                MockFunctions::mqi_outcome_ok(cc, rc);
            })
            .once();
        });

        let sub = Subscription {
            handle: 1.into(),
            connection: qm,
            close_options: values::MQCO::default(),
        };

        assert_eq!(sub.request_retained(&()).warn_as_error()?, 5);

        Ok(())
    }
}
