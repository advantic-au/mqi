use super::option;
use crate::{
    MqaiLibrary, Object,
    bag::{Bag, BagDrop},
    connection::AsConnection,
    types,
};

// use super::{Bag, BagDrop};

#[derive(Debug, Clone, Copy)]
pub struct OptionsBag<'a, B: BagDrop, L: MqaiLibrary>(&'a Bag<B, L>);
#[derive(Debug, Clone, Copy)]
pub struct ReplyObject<'a, C: AsConnection>(&'a Object<C>);
#[derive(Debug, Clone, Copy)]
pub struct AdminObject<'a, C: AsConnection>(&'a Object<C>);

impl<'a, B: BagDrop, L: MqaiLibrary> option::ExecuteOption<'a> for OptionsBag<'a, B, L> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.options.replace(self.0.handle());
    }
}

impl<'a, C: AsConnection> option::ExecuteOption<'a> for ReplyObject<'a, C> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl<'a, C: AsConnection> option::ExecuteOption<'a> for AdminObject<'a, C> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl option::ExecuteOption<'_> for types::MQCMD {
    fn apply_param(&self, param: &mut option::ExecuteParam) {
        param.command = *self;
    }
}
