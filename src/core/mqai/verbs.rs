use std::mem::size_of_val;
use std::ptr;

use libmqm_sys::MQAI;

use crate::core::{self, mqai, Library, MQFunctions, MQMDType};
use crate::sys;
use crate::{ResultComp, ResultErr};
use core::{MQIOutcome, MQIOutcomeVoid};

use super::{BagHandle, Filter, Op};

#[cfg(feature = "tracing")]
use {core::tracing_outcome, tracing::instrument};

impl<L: Library> MQFunctions<L>
where
    L::MQ: MQAI,
{
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_create_bag(&self, options: sys::MQLONG) -> ResultErr<BagHandle> {
        let mut outcome: MQIOutcome<BagHandle> = MQIOutcome::with_verb("mqCreateBag");
        unsafe {
            self.0
                .mqCreateBag(options, outcome.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_bag(&self, bag: &mut BagHandle) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqDeleteBag");
        unsafe {
            self.0
                .mqDeleteBag(bag.mut_raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_inquiry(&self, bag: &BagHandle, selector: sys::MQLONG) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInquiry");
        unsafe {
            self.0
                .mqAddInquiry(bag.raw_handle(), selector, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_delete_item(&self, bag: &BagHandle, selector: sys::MQLONG, index: sys::MQLONG) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqDeleteItem");
        unsafe {
            self.0
                .mqDeleteItem(bag.raw_handle(), selector, index, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer(&self, bag: &BagHandle, selector: sys::MQLONG, value: sys::MQLONG) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInteger");
        unsafe {
            self.0
                .mqAddInteger(bag.raw_handle(), selector, value, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer_filter(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<sys::MQLONG>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddIntegerFilter");
        unsafe {
            self.0.mqAddIntegerFilter(
                bag.raw_handle(),
                selector,
                value,
                op_value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_integer64(&self, bag: &BagHandle, selector: sys::MQLONG, value: sys::MQINT64) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddInteger64");
        unsafe {
            self.0
                .mqAddInteger64(bag.raw_handle(), selector, value, &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_string<T: ?Sized + std::fmt::Debug>(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        value: &T,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddString");
        let Self(lib) = self;
        unsafe {
            lib.mqAddString(
                bag.raw_handle(),
                selector,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
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
        selector: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<&T>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddStringFilter");
        unsafe {
            self.0.mqAddStringFilter(
                bag.raw_handle(),
                selector,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
                op_value,
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
        selector: sys::MQLONG,
        value: &T,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddByteString");
        unsafe {
            self.0.mqAddByteString(
                bag.raw_handle(),
                selector,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
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
        selector: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<&T>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddByteStringFilter");
        unsafe {
            self.0.mqAddByteStringFilter(
                bag.raw_handle(),
                selector,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
                op_value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_add_bag(&self, bag: &BagHandle, selector: sys::MQLONG, to_add: &BagHandle) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqAddBag");
        unsafe {
            self.0.mqAddBag(
                bag.raw_handle(),
                selector,
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
    pub fn mq_set_integer(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: sys::MQLONG,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetInteger");
        unsafe {
            self.0.mqSetInteger(
                bag.raw_handle(),
                selector,
                index,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<sys::MQLONG>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetIntegerFilter");
        unsafe {
            self.0.mqSetIntegerFilter(
                bag.raw_handle(),
                selector,
                index,
                value,
                op_value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_set_integer64(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: sys::MQINT64,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetInteger64");
        unsafe {
            self.0.mqSetInteger64(
                bag.raw_handle(),
                selector,
                index,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &T,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetString");
        unsafe {
            self.0.mqSetString(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<&T>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetStringFilter");
        unsafe {
            self.0.mqSetStringFilter(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
                op_value,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &T,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetByteString");
        unsafe {
            self.0.mqSetByteString(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        Filter {
            value,
            operator: Op(op_value),
        }: Filter<&T>,
    ) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqSetByteStringFilter");
        unsafe {
            self.0.mqSetByteStringFilter(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of!(*value).cast_mut().cast(),
                op_value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_integer(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        index: sys::MQLONG,
    ) -> ResultErr<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqInquireInteger");
        unsafe {
            self.0.mqInquireInteger(
                bag.raw_handle(),
                selector,
                index,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
    ) -> ResultErr<Filter<sys::MQLONG>> {
        let mut outcome = MQIOutcome::<Filter<sys::MQLONG>>::with_verb("mqInquireIntegerFilter");
        unsafe {
            self.0.mqInquireIntegerFilter(
                bag.raw_handle(),
                selector,
                index,
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
    pub fn mq_inquire_integer64(
        &self,
        bag: &BagHandle,
        selector: sys::MQLONG,
        index: sys::MQLONG,
    ) -> ResultErr<sys::MQINT64> {
        let mut outcome = MQIOutcome::with_verb("mqInquireInteger64");
        unsafe {
            self.0.mqInquireInteger64(
                bag.raw_handle(),
                selector,
                index,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &mut T,
    ) -> ResultErr<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqInquireByteString");
        unsafe {
            self.0.mqInquireByteString(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*value).cast(),
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &mut T,
    ) -> ResultErr<(sys::MQLONG, sys::MQLONG)> {
        let mut outcome = MQIOutcome::with_verb("mqInquireString");
        let (length, ccsid) = &mut outcome.value;
        unsafe {
            self.0.mqInquireString(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*value).cast(),
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &mut T,
    ) -> ResultErr<(sys::MQLONG, sys::MQLONG, sys::MQLONG)> {
        let mut outcome = MQIOutcome::with_verb("mqInquireStringFilter");
        let (length, ccsid, operator) = &mut outcome.value;
        unsafe {
            self.0.mqInquireStringFilter(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*value).cast(),
                length,
                ccsid,
                operator,
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
        selector: sys::MQLONG,
        index: sys::MQLONG,
        value: &mut T,
    ) -> ResultErr<(sys::MQLONG, sys::MQLONG)> {
        let mut outcome = MQIOutcome::with_verb("mqInquireByteStringFilter");
        let (length, operator) = &mut outcome.value;
        unsafe {
            self.0.mqInquireByteStringFilter(
                bag.raw_handle(),
                selector,
                index,
                size_of_val(value)
                    .try_into()
                    .expect("value length exceeds maximum positive MQLONG"),
                ptr::addr_of_mut!(*value).cast(),
                length,
                operator,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_inquire_bag(&self, bag: &BagHandle, selector: sys::MQLONG, index: sys::MQLONG) -> ResultErr<BagHandle> {
        let mut outcome = MQIOutcome::<BagHandle>::with_verb("mqInquireBag");
        unsafe {
            self.0.mqInquireBag(
                bag.raw_handle(),
                selector,
                index,
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
    pub fn mq_count_items(&self, bag: &BagHandle, selector: sys::MQLONG) -> ResultErr<sys::MQLONG> {
        let mut outcome = MQIOutcome::with_verb("mqCountItems");
        unsafe {
            self.0.mqCountItems(
                bag.raw_handle(),
                selector,
                &mut outcome.value,
                &mut outcome.cc.0,
                &mut outcome.rc.0,
            );
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_execute(
        &self,
        handle: &core::ConnectionHandle,
        command: sys::MQLONG,
        options: Option<&BagHandle>,
        admin: &BagHandle,
        response: &BagHandle,
        admin_q: Option<&core::ObjectHandle>,
        response_q: Option<&core::ObjectHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqExecute");
        unsafe {
            self.0.mqExecute(
                handle.raw_handle(),
                command,
                options.unwrap_or(&mqai::MQHB_NONE).raw_handle(),
                admin.raw_handle(),
                response.raw_handle(),
                admin_q.unwrap_or(&core::MQHO_NONE).raw_handle(),
                response_q.unwrap_or(&core::MQHO_NONE).raw_handle(),
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
    pub fn mq_clear_bag(&self, bag: &BagHandle) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqClearBag");
        unsafe {
            self.0
                .mqClearBag(bag.raw_handle(), &mut outcome.cc.0, &mut outcome.rc.0);
        }
        #[cfg(feature = "tracing")]
        tracing_outcome(&outcome);
        outcome.into()
    }

    /// Reduces the number of user items in a user bag to the specified value, by deleting user items from the end of the bag
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(self)))]
    pub fn mq_truncate_bag(&self, bag: &BagHandle, count: sys::MQLONG) -> ResultErr<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqTruncateBag");
        unsafe {
            self.0
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
        mqmd: &mut MQMDType,
        gmo: &mut sys::MQGMO,
        bag: Option<&BagHandle>,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqGetBag");
        unsafe {
            self.0.mqGetBag(
                handle.raw_handle(),
                object.raw_handle(),
                mqmd.as_mut_ptr(),
                ptr::addr_of_mut!(*gmo).cast(),
                bag.unwrap_or(&mqai::MQHB_NONE).raw_handle(),
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
        mqmd: &mut MQMDType,
        pmo: &mut sys::MQPMO,
        bag: &BagHandle,
    ) -> ResultComp<()> {
        let mut outcome = MQIOutcomeVoid::with_verb("mqPutBag");
        unsafe {
            self.0.mqPutBag(
                handle.raw_handle(),
                object.raw_handle(),
                mqmd.as_mut_ptr(),
                ptr::addr_of_mut!(*pmo).cast(),
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
    use super::*;

    #[test]
    fn create_bag() {
        let linked = MQFunctions::linked();
        let mut bag = linked
            .mq_create_bag(sys::MQCBO_COMMAND_BAG)
            .expect("Failed to create MQ BAG");
        linked.mq_delete_bag(&mut bag).expect("Failed to delete MQ BAG");
    }

    #[test]
    fn add_bag() {
        let mq_lib = MQFunctions::linked();
        let mut bag = mq_lib
            .mq_create_bag(sys::MQCBO_GROUP_BAG)
            .expect("Failed to create MQ BAG");
        let bag_attached = mq_lib
            .mq_create_bag(sys::MQCBO_GROUP_BAG)
            .expect("Failed to create MQ BAG");
        let mut wally: [sys::MQCHAR; 3] = [1, 2, 3];
        mq_lib.mq_add_bag(&bag, 0, &bag_attached).expect("Failed to add bag");
        dbg!(mq_lib.mq_inquire_bag(&bag, 0, 0)).expect("Failed to inquire embedded bag");
        dbg!(mq_lib.mq_add_integer(&bag_attached, 0, 999)).expect("BLA");
        dbg!(mq_lib.mq_add_string(&bag_attached, 1, &wally)).expect("BLA");

        wally[0] = 9;

        //dbg!(mq_lib.mq_add_string(&bag_attached, 2, "hello".as_bytes())).expect("BLA2");
        let mut data = Vec::<u8>::with_capacity(page_size::get());
        let (length, ..) =
            dbg!(mq_lib.mq_inquire_string(&bag_attached, 1, 0, data.spare_capacity_mut())).expect("BLA2");
        unsafe {
            data.set_len(
                length
                    .try_into()
                    .expect("length returned by mq_inquire_string is a negative number"),
            );
        }
        dbg!(data);
        //mq_lib.mq_delete_bag(&mut bag_attached).expect("Failed to delete MQ BAG");
        let r = dbg!(mq_lib.mq_inquire_bag(&bag, 0, 0)).expect("Failed to inquire bag");
        dbg!(mq_lib.mq_inquire_integer(&r, 0, 0)).expect("Failed to retrieve it again");
        dbg!(&bag);
        dbg!(&bag_attached);
        mq_lib.mq_delete_bag(&mut bag).expect("Failed to delete MQ BAG");
    }
}
