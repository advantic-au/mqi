use crate::{
    handle::{BagHandle, ObjectHandle},
    macros::all_option_tuples,
    types::MQCMD,
};

/// Parameters populated by the MQAI `mqExecute` operation.
#[derive(Debug, Default)]
pub struct ExecuteParam<'a> {
    /// MQAI command to execute.
    pub command: MQCMD,
    /// Optional input bag containing command options.
    pub options: Option<&'a BagHandle>,
    /// Optional object handle identifying the administration target.
    pub admin_object: Option<&'a ObjectHandle>,
    /// Optional object handle that receives command output.
    pub reply_object: Option<&'a ObjectHandle>,
}

/// A trait that manipulates the parameters to the [`mqExecute`](`::libmqm_sys::mqai::mqExecute`) function
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `ExecuteOption` so it can't be used as an argument for MQI mqExecute"
)]
pub trait ExecuteOption<'a> {
    /// Apply this setting to the MQAI command and its bag/object parameters.
    fn apply_param(&self, param: &mut ExecuteParam<'a>);
}

all_option_tuples!('e, ExecuteOption, ExecuteParam<'e>);
