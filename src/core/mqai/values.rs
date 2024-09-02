use crate::{define_mqvalue, impl_default_mqvalue, mapping, sys, ConstLookup};

define_mqvalue!(pub MQIND, mapping::MQIND_CONST);
define_mqvalue!(pub MQQT, mapping::MQQT_CONST);
define_mqvalue!(pub MQAT, mapping::MQAT_CONST);
define_mqvalue!(pub MQCMD, mapping::MQCMD_CONST);
define_mqvalue!(pub MQCFOP, mapping::MQCFOP_CONST);
define_mqvalue!(pub MqaiSelector, MqaiSelectorLookup);
impl_default_mqvalue!(MQIND, sys::MQIND_NONE);
/*

MQAI selector constant lookup is complex... thanks to this - no less than 8 different constant sets.
https://www.ibm.com/docs/en/ibm-mq/latest?topic=reference-mqai-selectors

It would be more efficient to generate one large set as part of the build process, but this will do for now.

*/

struct MqaiSelectorLookup;
impl ConstLookup for MqaiSelectorLookup {
    fn by_value(&self, value: sys::MQLONG) -> impl Iterator<Item = &str> {
        mapping::MQIA_CONST
            .by_value(value)
            .chain(mapping::MQCA_CONST.by_value(value))
            .chain(mapping::MQIACF_CONST.by_value(value))
            .chain(mapping::MQCACF_CONST.by_value(value))
            .chain(mapping::MQIACH_CONST.by_value(value))
            .chain(mapping::MQCACH_CONST.by_value(value))
            .chain(mapping::MQIASY_CONST.by_value(value))
            .chain(mapping::MQHA_CONST.by_value(value))
    }

    fn by_name(&self, name: &str) -> Option<sys::MQLONG> {
        mapping::MQIA_CONST
            .by_name(name)
            .or_else(|| mapping::MQCA_CONST.by_name(name))
            .or_else(|| mapping::MQIACF_CONST.by_name(name))
            .or_else(|| mapping::MQCACF_CONST.by_name(name))
            .or_else(|| mapping::MQIACH_CONST.by_name(name))
            .or_else(|| mapping::MQCACH_CONST.by_name(name))
            .or_else(|| mapping::MQIASY_CONST.by_name(name))
            .or_else(|| mapping::MQHA_CONST.by_name(name))
    }

    fn all(&self) -> impl Iterator<Item = crate::ConstantItem> {
        mapping::MQIA_CONST
            .all()
            .chain(mapping::MQCA_CONST.all())
            .chain(mapping::MQIACF_CONST.all())
            .chain(mapping::MQCACF_CONST.all())
            .chain(mapping::MQIACH_CONST.all())
            .chain(mapping::MQCACH_CONST.all())
            .chain(mapping::MQIASY_CONST.all())
            .chain(mapping::MQHA_CONST.all())
    }
}
