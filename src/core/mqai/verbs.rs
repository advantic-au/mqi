use std::mem::size_of_val;
use std::ptr;

use libmqm_sys::Mqai;

use crate::core::{CCSID, ConnectionHandle, Library, MqFunctions, MqInqError, MqiOutcome, MqiOutcomeVoid, ObjectHandle, WriteRaw};
use crate::{Error, ResultCompErr, MQMD};
use crate::{sys, constants, ResultComp};
use crate::types::{Selector, MQCBO, MQCFOP, MQCMD, MQIND, MQITEM};
use super::{BagHandle, Filter};

#[cfg(feature = "tracing")]
use {crate::core::tracing_outcome, tracing::instrument};

impl<L: Library<MQ: Mqai>> MqFunctions<L> {
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_create_bag(&self, options: MQCBO) -> ResultComp<BagHandle> {
        let mut outcome = MqiOutcome::<BagHandle>::with_verb("mqCreateBag");
        unsafe {
            self.0.lib().mqCreateBag(
                options.0,
                outcome.mut_raw_handle(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_bag(&self, bag: &mut BagHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqDeleteBag");
        unsafe {
            self.0
                .lib()
                .mqDeleteBag(bag.mut_raw_handle(), &raw mut outcome.cc.0, &raw mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_inquiry(&self, bag: &BagHandle, selector: Selector) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddInquiry");
        unsafe {
            self.0
                .lib()
                .mqAddInquiry(bag.raw_handle(), selector.0, &raw mut outcome.cc.0, &raw mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_item(&self, bag: &BagHandle, selector: Selector, index: MQIND) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqDeleteItem");
        unsafe {
            self.0.lib().mqDeleteItem(
                bag.raw_handle(),
                selector.0,
                index.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer(&self, bag: &BagHandle, selector: Selector, value: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddInteger");
        unsafe {
            self.0.lib().mqAddInteger(
                bag.raw_handle(),
                selector.0,
                value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        Filter { value, operator }: Filter<sys::MQLONG>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddIntegerFilter");
        unsafe {
            self.0.lib().mqAddIntegerFilter(
                bag.raw_handle(),
                selector.0,
                value,
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer64(&self, bag: &BagHandle, selector: Selector, value: sys::MQINT64) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddInteger64");
        unsafe {
            self.0.lib().mqAddInteger64(
                bag.raw_handle(),
                selector.0,
                value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_string(&self, bag: &BagHandle, selector: Selector, value: &[sys::MQCHAR]) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddString");
        unsafe {
            self.0.lib().mqAddString(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        Filter { value, operator }: Filter<&[sys::MQCHAR]>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddStringFilter");
        unsafe {
            self.0.lib().mqAddStringFilter(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_byte_string(&self, bag: &BagHandle, selector: Selector, value: &[sys::MQBYTE]) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddByteString");
        unsafe {
            self.0.lib().mqAddByteString(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_byte_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        Filter { value, operator }: Filter<&[sys::MQBYTE]>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddByteStringFilter");
        unsafe {
            self.0.lib().mqAddByteStringFilter(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_bag(&self, bag: &BagHandle, selector: Selector, to_add: &BagHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqAddBag");
        unsafe {
            self.0.lib().mqAddBag(
                bag.raw_handle(),
                selector.0,
                to_add.raw_handle(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer(&self, bag: &BagHandle, selector: Selector, index: MQIND, value: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetInteger");
        unsafe {
            self.0.lib().mqSetInteger(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        Filter { value, operator }: Filter<sys::MQLONG>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetIntegerFilter");
        unsafe {
            self.0.lib().mqSetIntegerFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer64(&self, bag: &BagHandle, selector: Selector, index: MQIND, value: sys::MQINT64) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetInteger64");
        unsafe {
            self.0.lib().mqSetInteger64(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_string(&self, bag: &BagHandle, selector: Selector, index: MQIND, value: &[sys::MQCHAR]) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetString");
        unsafe {
            self.0.lib().mqSetString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        Filter { value, operator }: Filter<&[sys::MQCHAR]>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetStringFilter");
        unsafe {
            self.0.lib().mqSetStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_byte_string(&self, bag: &BagHandle, selector: Selector, index: MQIND, value: &[sys::MQBYTE]) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetByteString");
        unsafe {
            self.0.lib().mqSetByteString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_byte_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        Filter { value, operator }: Filter<&[sys::MQBYTE]>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqSetByteStringFilter");
        unsafe {
            self.0.lib().mqSetByteStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer(&self, bag: &BagHandle, selector: Selector, index: MQIND) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("mqInquireInteger");
        unsafe {
            self.0.lib().mqInquireInteger(
                bag.raw_handle(),
                selector.0,
                index.0,
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
    ) -> ResultComp<Filter<sys::MQLONG>> {
        let mut outcome = MqiOutcome::new("mqInquireIntegerFilter", Filter::new(0, MQCFOP(0)));
        unsafe {
            self.0.lib().mqInquireIntegerFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                &raw mut outcome.value.value,
                &raw mut outcome.value.operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer64(&self, bag: &BagHandle, selector: Selector, index: MQIND) -> ResultComp<sys::MQINT64> {
        let mut outcome = MqiOutcome::with_verb("mqInquireInteger64");
        unsafe {
            self.0.lib().mqInquireInteger64(
                bag.raw_handle(),
                selector.0,
                index.0,
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_byte_string(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        value: &mut (impl WriteRaw<sys::MQBYTE> + ?Sized),
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("mqInquireByteString");
        unsafe {
            self.0.lib().mqInquireByteString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_string(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        value: &mut (impl WriteRaw<sys::MQCHAR> + ?Sized),
    ) -> ResultComp<(sys::MQLONG, CCSID)> {
        let mut outcome = MqiOutcome::<(sys::MQLONG, CCSID)>::with_verb("mqInquireString");
        let (length, ccsid) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                &raw mut ccsid.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        value: &mut (impl WriteRaw<sys::MQCHAR> + ?Sized),
    ) -> ResultComp<(sys::MQLONG, CCSID, MQCFOP)> {
        let mut outcome = MqiOutcome::new("mqInquireStringFilter", (-1, CCSID(0), MQCFOP(0)));
        let (length, ccsid, operator) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                &raw mut ccsid.0,
                &raw mut operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_byte_string_filter(
        &self,
        bag: &BagHandle,
        selector: Selector,
        index: MQIND,
        value: &mut (impl WriteRaw<sys::MQBYTE> + ?Sized),
    ) -> ResultComp<(sys::MQLONG, MQCFOP)> {
        let mut outcome = MqiOutcome::new("mqInquireByteStringFilter", (-1, MQCFOP(0)));
        let (length, operator) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireByteStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length should not exceed maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                &raw mut operator.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_bag(&self, bag: &BagHandle, selector: Selector, index: MQIND) -> ResultComp<BagHandle> {
        let mut outcome = MqiOutcome::<BagHandle>::with_verb("mqInquireBag");
        unsafe {
            self.0.lib().mqInquireBag(
                bag.raw_handle(),
                selector.0,
                index.0,
                outcome.mut_raw_handle(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_count_items(&self, bag: &BagHandle, selector: Selector) -> ResultComp<sys::MQLONG> {
        let mut outcome = MqiOutcome::with_verb("mqCountItems");
        unsafe {
            self.0.lib().mqCountItems(
                bag.raw_handle(),
                selector.0,
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[expect(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_execute(
        &self,
        handle: ConnectionHandle,
        command: MQCMD,
        options: Option<&BagHandle>,
        admin: &BagHandle,
        response: &BagHandle,
        admin_q: Option<&ObjectHandle>,
        response_q: Option<&ObjectHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqExecute");
        unsafe {
            self.0.lib().mqExecute(
                handle.raw_handle(),
                command.0,
                options.map_or(sys::MQHB_NONE, |h| h.raw_handle()),
                admin.raw_handle(),
                response.raw_handle(),
                admin_q.map_or(sys::MQHO_NONE, |h| h.raw_handle()),
                response_q.map_or(sys::MQHO_NONE, |h| h.raw_handle()),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Deletes all user items from the bag, and resets system items to their initial values
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_clear_bag(&self, bag: &BagHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqClearBag");
        unsafe {
            self.0
                .lib()
                .mqClearBag(bag.raw_handle(), &raw mut outcome.cc.0, &raw mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Reduces the number of user items in a user bag to the specified value, by deleting user items from the end of the bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_truncate_bag(&self, bag: &BagHandle, count: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqTruncateBag");
        unsafe {
            self.0
                .lib()
                .mqTruncateBag(bag.raw_handle(), count, &raw mut outcome.cc.0, &raw mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Removes a message from the specified queue and converts the message data into a data bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, bag)))]
    pub fn mq_get_bag(
        &self,
        handle: ConnectionHandle,
        object: &ObjectHandle,
        mqmd: &mut impl MQMD,
        gmo: &mut sys::MQGMO,
        bag: Option<&BagHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqGetBag");
        unsafe {
            self.0.lib().mqGetBag(
                handle.raw_handle(),
                object.raw_handle(),
                ptr::from_mut(mqmd).cast(),
                ptr::from_mut(gmo).cast(),
                bag.map_or(sys::MQHB_NONE, |h| h.raw_handle()),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Converts the contents of the specified bag into a PCF message and sends the message to the specified queue.
    ///
    /// # Safety
    /// Consumers of [`mq_put_bag`](MqFunctions::mq_put_bag) must ensure the [`MQPMO`](sys::MQPMO) structure is populated with valid pointers and offsets
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub unsafe fn mq_put_bag(
        &self,
        handle: ConnectionHandle,
        object: &ObjectHandle,
        mqmd: &mut impl MQMD,
        pmo: &mut sys::MQPMO,
        bag: &BagHandle,
    ) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqPutBag");
        unsafe {
            self.0.lib().mqPutBag(
                handle.raw_handle(),
                object.raw_handle(),
                ptr::from_mut(mqmd).cast(),
                ptr::from_mut(pmo).cast(),
                bag.raw_handle(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Convert the bag into a PCF message in the supplied buffer
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, buffer)))]
    pub fn mq_bag_to_buffer(
        &self,
        options_bag: &BagHandle,
        data_bag: &BagHandle,
        buffer: Option<&mut (impl WriteRaw<sys::MQBYTE> + ?Sized)>,
    ) -> ResultCompErr<sys::MQLONG, MqInqError> {
        let mut outcome = MqiOutcome::with_verb("mqBagToBuffer");

        let (buf, len) = buffer.map_or((ptr::null_mut(), 0), |buffer| {
            (
                ptr::from_mut(buffer).cast(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length should not exceed maximum positive MQLONG"),
            )
        });
        unsafe {
            self.0.lib().mqBagToBuffer(
                options_bag.raw_handle(),
                data_bag.raw_handle(),
                len,
                buf,
                &raw mut outcome.value,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        match outcome.rc {
            constants::MQRC_BUFFER_LENGTH_ERROR => {
                Err(MqInqError::Length(outcome.value, Error(outcome.cc, outcome.verb, outcome.rc)))
            }
            _ => outcome.into(),
        }
    }

    /// Convert the supplied buffer into bag form
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, buffer)))]
    pub fn mq_buffer_to_bag(&self, options_bag: &BagHandle, buffer: &[sys::MQBYTE], data_bag: &mut BagHandle) -> ResultComp<()> {
        let mut outcome = MqiOutcomeVoid::with_verb("mqBufferToBag");
        unsafe {
            self.0.lib().mqBufferToBag(
                options_bag.raw_handle(),
                size_of_val(buffer)
                    .try_into()
                    .expect("buffer length should not exceed maximum positive MQLONG"),
                ptr::from_ref(buffer).cast_mut().cast(),
                data_bag.raw_handle(),
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Return information about a specified item in a bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_item_info(&self, bag: &BagHandle, selector: Selector, index: MQIND) -> ResultComp<(Selector, MQITEM)> {
        let mut outcome = MqiOutcome::new("mqInquireItemInfo", (Selector(-1), MQITEM(-1)));

        unsafe {
            self.0.lib().mqInquireItemInfo(
                bag.raw_handle(),
                selector.0,
                index.0,
                &raw mut outcome.value.0.0,
                &raw mut outcome.value.1.0,
                &raw mut outcome.cc.0,
                &raw mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }
}

#[cfg(all(test, any(feature = "link", feature = "dlopen2")))]
mod tests {
    use std::error::Error;

    use crate::{test::mq_library, MqChar, prelude::*};

    use super::*;

    #[test]
    fn inquire_integer() {
        let mq_lib = MqFunctions(mq_library());
        let mut bag = mq_lib
            .mq_create_bag(constants::MQCBO_COMMAND_BAG)
            .warn_as_error()
            .expect("creation of MQ bag should not fail");

        // MQIASY_BAG_OPTIONS, index 0 should exist
        let options = mq_lib
            .mq_inquire_integer(&bag, constants::MQIASY_BAG_OPTIONS.into(), MQIND(0))
            .warn_as_error()
            .expect("options retrieval should not fail");
        assert_eq!(MQCBO(options), constants::MQCBO_COMMAND_BAG);

        // MQIASY_BAG_OPTIONS, index 1 should not exist
        mq_lib
            .mq_inquire_integer(&bag, constants::MQIASY_BAG_OPTIONS.into(), MQIND(1))
            .warn_as_error()
            .expect_err("options retrieval should fail");

        mq_lib
            .mq_delete_bag(&mut bag)
            .warn_as_error()
            .expect("deletion of MQ bag should not fail");
    }

    #[test]
    fn inquire_item_info() {
        let mq_lib = MqFunctions(mq_library());
        let mut bag = mq_lib
            .mq_create_bag(constants::MQCBO_COMMAND_BAG)
            .warn_as_error()
            .expect("creation of MQ bag should not fail");
        let (sel, item) = mq_lib
            .mq_inquire_item_info(&bag, constants::MQIASY_BAG_OPTIONS.into(), MQIND(0))
            .warn_as_error()
            .expect("info of item should not fail");
        assert_eq!(item, constants::MQITEM_INTEGER);
        assert_eq!(sel, Selector(sys::MQIASY_BAG_OPTIONS));
        mq_lib
            .mq_inquire_item_info(&bag, constants::MQIASY_BAG_OPTIONS.into(), MQIND(1))
            .expect_err("index 1 should not exist");
        mq_lib
            .mq_delete_bag(&mut bag)
            .warn_as_error()
            .expect("deletion of MQ bag should not fail");
    }

    #[test]
    fn bag_buffer() {
        let mq_lib = MqFunctions(mq_library());
        let mut bag = mq_lib
            .mq_create_bag(constants::MQCBO_NONE)
            .warn_as_error()
            .expect("creation of MQ bag should not fail");
        let mut buffer = vec![0u8; 2 * 1024 * 1024]; // 2Mb buffer
        mq_lib
            .mq_add_integer(&bag, Selector(1), 99)
            .warn_as_error()
            .expect("mq_set_integer should succeed");
        let length = mq_lib
            .mq_bag_to_buffer(&BagHandle::from(sys::MQHB_NONE), &bag, Some(buffer.as_mut_slice()))
            .warn_as_error()
            .expect("mqBagToBuffer should succeed");
        let bag_buffer = &buffer[..length.try_into().expect("returned length should convert to usize")];
        let mut bag_target = mq_lib
            .mq_create_bag(constants::MQCBO_NONE)
            .warn_as_error()
            .expect("creation of MQ bag should succeed");
        mq_lib
            .mq_buffer_to_bag(&BagHandle::from(sys::MQHB_NONE), bag_buffer, &mut bag_target)
            .warn_as_error()
            .expect("mqBufferToBag should succeed");
        let target_int = mq_lib
            .mq_inquire_integer(&bag_target, Selector(1), MQIND(0))
            .warn_as_error()
            .expect("options retrieval should succeed");
        assert_eq!(target_int, 99);

        mq_lib
            .mq_delete_bag(&mut bag)
            .warn_as_error()
            .expect("deletion of MQ bag should succeed");
    }

    #[test]
    fn add_bag() -> Result<(), Box<dyn Error>> {
        let mq_lib = MqFunctions(mq_library());
        let mut bag = mq_lib.mq_create_bag(constants::MQCBO_GROUP_BAG)?;
        let bag_attached = mq_lib.mq_create_bag(constants::MQCBO_GROUP_BAG)?;
        let mut wally: MqChar<3> = [1, 2, 3];
        mq_lib
            .mq_add_bag(&bag, Selector(0), &bag_attached)
            .warn_as_error()
            .expect("adding to a bag should succeed");
        dbg!(mq_lib.mq_inquire_bag(&bag, Selector(0), MQIND(0))).warn_as_error()?;
        dbg!(mq_lib.mq_add_integer(&bag_attached, Selector(0), 999)).warn_as_error()?;
        dbg!(mq_lib.mq_add_string(&bag_attached, Selector(1), &wally)).warn_as_error()?;

        wally[0] = 9;

        //dbg!(mq_lib.mq_add_string(&bag_attached, 2, "hello".as_bytes())).expect("BLA2");
        let mut data = vec![0i8; 4096];
        let (length, ..) = dbg!(mq_lib.mq_inquire_string(&bag_attached, Selector(1), MQIND(0), data.as_mut_slice()))
            .warn_as_error()
            .expect("BLA2");
        let data = &data[..length.try_into().expect("length converts to usize")];
        dbg!(data);
        //mq_lib.mq_delete_bag(&mut bag_attached).expect("Failed to delete MQ BAG");
        let r = dbg!(mq_lib.mq_inquire_bag(&bag, Selector(0), MQIND(0))).warn_as_error()?;
        dbg!(mq_lib.mq_inquire_integer(&r, Selector(0), MQIND(0))).warn_as_error()?;
        dbg!(&bag);
        dbg!(&bag_attached);
        mq_lib.mq_delete_bag(&mut bag).warn_as_error()?;

        Ok(())
    }

    #[test]
    fn selector() {
        assert_eq!(format!("{}", Selector(0)), "0");
        assert_eq!(format!("{}", Selector(sys::MQCA_ALTERATION_TIME)), "MQCA_ALTERATION_TIME");
        assert_eq!(format!("{}", Selector(sys::MQIACF_INQUIRY)), "MQIACF_INQUIRY");
        assert_eq!(format!("{}", Selector(sys::MQSEL_ANY_SELECTOR)), "-30001");
    }
}
