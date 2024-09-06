use std::mem::size_of_val;
use std::ptr;

use libmqm_sys::MQAI;

use crate::core::values::MQCBO;
use crate::core::{Library, MQFunctions, MQIOutcome, MQIOutcomeVoid};
use crate::{core, MQMD};
use crate::{sys, ResultComp};

use super::values::{MqaiSelector, MQCFOP, MQCMD, MQIND};
use super::{BagHandle, Filter};

#[cfg(feature = "tracing")]
use {core::tracing_outcome, tracing::instrument};

impl<L: Library<MQ: MQAI>> MQFunctions<L> {
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_create_bag(&self, options: MQCBO) -> ResultComp<BagHandle> {
        let mut outcome = MQIOutcome::<BagHandle>::with_verb("mqCreateBag");
        unsafe {
            self.0
                .lib()
                .mqCreateBag(options.0, outcome.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_bag(&self, bag: &mut BagHandle) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqDeleteBag");
        unsafe {
            self.0
                .lib()
                .mqDeleteBag(bag.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_inquiry(&self, bag: &BagHandle, selector: MqaiSelector) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInquiry");
        unsafe {
            self.0
                .lib()
                .mqAddInquiry(bag.raw_handle(), selector.0, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_item(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqDeleteItem");
        unsafe {
            self.0
                .lib()
                .mqDeleteItem(bag.raw_handle(), selector.0, index.0, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer(&self, bag: &BagHandle, selector: MqaiSelector, value: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInteger");
        unsafe {
            self.0
                .lib()
                .mqAddInteger(bag.raw_handle(), selector.0, value, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer_filter(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        Filter { value, operator }: Filter<sys::MQLONG>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddIntegerFilter");
        unsafe {
            self.0.lib().mqAddIntegerFilter(
                bag.raw_handle(),
                selector.0,
                value,
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer64(&self, bag: &BagHandle, selector: MqaiSelector, value: sys::MQINT64) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInteger64");
        unsafe {
            self.0
                .lib()
                .mqAddInteger64(bag.raw_handle(), selector.0, value, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_string<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddString");
        unsafe {
            self.0.lib().mqAddString(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_string_filter<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        Filter { value, operator }: Filter<&T>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddStringFilter");
        unsafe {
            self.0.lib().mqAddStringFilter(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_byte_string<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddByteString");
        unsafe {
            self.0.lib().mqAddByteString(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_byte_string_filter<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        Filter { value, operator }: Filter<&T>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddByteStringFilter");
        unsafe {
            self.0.lib().mqAddByteStringFilter(
                bag.raw_handle(),
                selector.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_bag(&self, bag: &BagHandle, selector: MqaiSelector, to_add: &BagHandle) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddBag");
        unsafe {
            self.0.lib().mqAddBag(
                bag.raw_handle(),
                selector.0,
                to_add.raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND, value: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetInteger");
        unsafe {
            self.0.lib().mqSetInteger(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
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
        selector: MqaiSelector,
        index: MQIND,
        Filter { value, operator }: Filter<sys::MQLONG>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetIntegerFilter");
        unsafe {
            self.0.lib().mqSetIntegerFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer64(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND, value: sys::MQINT64) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetInteger64");
        unsafe {
            self.0.lib().mqSetInteger64(
                bag.raw_handle(),
                selector.0,
                index.0,
                value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_string<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetString");
        unsafe {
            self.0.lib().mqSetString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_string_filter<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        Filter { value, operator }: Filter<&T>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetStringFilter");
        unsafe {
            self.0.lib().mqSetStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_byte_string<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &T,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetByteString");
        unsafe {
            self.0.lib().mqSetByteString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_byte_string_filter<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        Filter { value, operator }: Filter<&T>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetByteStringFilter");
        unsafe {
            self.0.lib().mqSetByteStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_ref(value).cast_mut().cast(),
                operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqInquireInteger");
        unsafe {
            self.0.lib().mqInquireInteger(
                bag.raw_handle(),
                selector.0,
                index.0,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
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
        selector: MqaiSelector,
        index: MQIND,
    ) -> ResultComp<Filter<sys::MQLONG>> {
        let mut outcome = MQIOutcome::new("mqInquireIntegerFilter", Filter::new(0, MQCFOP(0)));
        unsafe {
            self.0.lib().mqInquireIntegerFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                &mut outcome.value.value,
                &mut outcome.value.operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer64(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND) -> ResultComp<sys::MQINT64> {
        let mut outcome = MQIOutcome::with_verb("mqInquireInteger64");
        unsafe {
            self.0.lib().mqInquireInteger64(
                bag.raw_handle(),
                selector.0,
                index.0,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_byte_string<T: ?Sized>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &mut T,
    ) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqInquireByteString");
        unsafe {
            self.0.lib().mqInquireByteString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_string<T: ?Sized>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &mut T,
    ) -> ResultComp<(sys::MQLONG, sys::MQLONG)> {
        let mut outcome = MQIOutcome::with_verb("mqInquireString");
        let (length, ccsid) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireString(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                ccsid,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_string_filter<T: ?Sized>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &mut T,
    ) -> ResultComp<(sys::MQLONG, sys::MQLONG, MQCFOP)> {
        let mut outcome = MQIOutcome::new("mqInquireStringFilter", (-1, 0, MQCFOP(0)));
        let (length, ccsid, operator) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                ccsid,
                &mut operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value, self)))]
    pub fn mq_inquire_byte_string_filter<T: ?Sized>(
        &self,
        bag: &BagHandle,
        selector: MqaiSelector,
        index: MQIND,
        value: &mut T,
    ) -> ResultComp<(sys::MQLONG, MQCFOP)> {
        let mut outcome = MQIOutcome::new("mqInquireByteStringFilter", (-1, MQCFOP(0)));
        let (length, operator) = &mut outcome.value;
        unsafe {
            self.0.lib().mqInquireByteStringFilter(
                bag.raw_handle(),
                selector.0,
                index.0,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::from_mut(value).cast(),
                length,
                &mut operator.0,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_bag(&self, bag: &BagHandle, selector: MqaiSelector, index: MQIND) -> ResultComp<BagHandle> {
        let mut outcome = MQIOutcome::<BagHandle>::with_verb("mqInquireBag");
        unsafe {
            self.0.lib().mqInquireBag(
                bag.raw_handle(),
                selector.0,
                index.0,
                outcome.mut_raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_count_items(&self, bag: &BagHandle, selector: MqaiSelector) -> ResultComp<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqCountItems");
        unsafe {
            self.0.lib().mqCountItems(
                bag.raw_handle(),
                selector.0,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
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
        handle: &core::ConnectionHandle,
        command: MQCMD,
        options: Option<&BagHandle>,
        admin: &BagHandle,
        response: &BagHandle,
        admin_q: Option<&core::ObjectHandle>,
        response_q: Option<&core::ObjectHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqExecute");
        unsafe {
            self.0.lib().mqExecute(
                handle.raw_handle(),
                command.0,
                options.map_or(sys::MQHB_NONE, |h| h.raw_handle()),
                admin.raw_handle(),
                response.raw_handle(),
                admin_q.map_or(sys::MQHO_NONE, |h| h.raw_handle()),
                response_q.map_or(sys::MQHO_NONE, |h| h.raw_handle()),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Deletes all user items from the bag, and resets system items to their initial values
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_clear_bag(&self, bag: &BagHandle) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqClearBag");
        unsafe {
            self.0
                .lib()
                .mqClearBag(bag.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Reduces the number of user items in a user bag to the specified value, by deleting user items from the end of the bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_truncate_bag(&self, bag: &BagHandle, count: sys::MQLONG) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqTruncateBag");
        unsafe {
            self.0
                .lib()
                .mqTruncateBag(bag.raw_handle(), count, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Removes a message from the specified queue and converts the message data into a data bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self, bag)))]
    pub fn mq_get_bag(
        &self,
        handle: &core::ConnectionHandle,
        object: &core::ObjectHandle,
        mqmd: &mut impl MQMD,
        gmo: &mut sys::MQGMO,
        bag: Option<&BagHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqGetBag");
        unsafe {
            self.0.lib().mqGetBag(
                handle.raw_handle(),
                object.raw_handle(),
                ptr::from_mut(mqmd).cast(),
                ptr::from_mut(gmo).cast(),
                bag.map_or(sys::MQHB_NONE, |h| h.raw_handle()),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    // Converts the contents of the specified bag into a PCF message and sends the message to the specified queue.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_put_bag(
        &self,
        handle: &core::ConnectionHandle,
        object: &core::ObjectHandle,
        mqmd: &mut impl MQMD,
        pmo: &mut sys::MQPMO,
        bag: &BagHandle,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqPutBag");
        unsafe {
            self.0.lib().mqPutBag(
                handle.raw_handle(),
                object.raw_handle(),
                ptr::from_mut(mqmd).cast(),
                ptr::from_mut(pmo).cast(),
                bag.raw_handle(),
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::ResultCompExt;

    use super::*;

    #[test]
    fn create_bag() {
        let linked = MQFunctions::linked();
        let mut bag = linked
            .mq_create_bag(MQCBO(sys::MQCBO_COMMAND_BAG))
            .expect("Failed to create MQ BAG");
        linked
            .mq_delete_bag(&mut bag)
            .warn_as_error()
            .expect("Failed to delete MQ BAG");
    }

    #[test]
    fn add_bag() -> Result<(), Box<dyn Error>> {
        let mq_lib = MQFunctions::linked();
        let mut bag = mq_lib.mq_create_bag(MQCBO(sys::MQCBO_GROUP_BAG))?;
        let bag_attached = mq_lib.mq_create_bag(MQCBO(sys::MQCBO_GROUP_BAG))?;
        let mut wally: [sys::MQCHAR; 3] = [1, 2, 3];
        mq_lib
            .mq_add_bag(&bag, MqaiSelector(0), &bag_attached)
            .warn_as_error()
            .expect("Failed to add bag");
        dbg!(mq_lib.mq_inquire_bag(&bag, MqaiSelector(0), MQIND(0))).warn_as_error()?;
        dbg!(mq_lib.mq_add_integer(&bag_attached, MqaiSelector(0), 999)).warn_as_error()?;
        dbg!(mq_lib.mq_add_string(&bag_attached, MqaiSelector(1), &wally)).warn_as_error()?;

        wally[0] = 9;

        //dbg!(mq_lib.mq_add_string(&bag_attached, 2, "hello".as_bytes())).expect("BLA2");
        let mut data = Vec::<u8>::with_capacity(page_size::get());
        let (length, ..) = dbg!(mq_lib.mq_inquire_string(&bag_attached, MqaiSelector(1), MQIND(0), data.spare_capacity_mut()))
            .warn_as_error()
            .expect("BLA2");
        unsafe {
            data.set_len(
                length
                    .try_into()
                    .expect("length returned by mq_inquire_string is a negative number"),
            );
        }
        dbg!(data);
        //mq_lib.mq_delete_bag(&mut bag_attached).expect("Failed to delete MQ BAG");
        let r = dbg!(mq_lib.mq_inquire_bag(&bag, MqaiSelector(0), MQIND(0))).warn_as_error()?;
        dbg!(mq_lib.mq_inquire_integer(&r, MqaiSelector(0), MQIND(0))).warn_as_error()?;
        dbg!(&bag);
        dbg!(&bag_attached);
        mq_lib.mq_delete_bag(&mut bag).warn_as_error()?;

        Ok(())
    }

    #[test]
    fn mqaiselector() {
        assert_eq!(format!("{}", MqaiSelector(0)), "0");
        assert_eq!(format!("{}", MqaiSelector(sys::MQCA_ALTERATION_TIME)), "MQCA_ALTERATION_TIME");
        assert_eq!(format!("{}", MqaiSelector(sys::MQIACF_INQUIRY)), "MQIACF_INQUIRY");
        assert_eq!(format!("{}", MqaiSelector(sys::MQSEL_ANY_SELECTOR)), "-30001");
    }
}
