pub(super) mod option {
    use crate::macros::all_option_tuples;

    use crate::BagHandle;
    use crate::ObjectHandle;
    use crate::types::MQCMD;

    #[derive(Debug, Default)]
    pub struct ExecuteParam<'a> {
        pub command: MQCMD,
        pub options: Option<&'a BagHandle>,
        pub admin_object: Option<&'a ObjectHandle>,
        pub reply_object: Option<&'a ObjectHandle>,
    }

    /// A trait that manipulates the parameters to the [`mqExecute`](`::libmqm_sys::mqai::mqExecute`) function
    #[diagnostic::on_unimplemented(
        message = "{Self} does not implement `ExecuteOption` so it can't be used as an argument for MQI mqExecute"
    )]
    pub trait ExecuteOption<'a> {
        fn apply_param(&self, param: &mut ExecuteParam<'a>);
    }

    all_option_tuples!('e, ExecuteOption, ExecuteParam<'e>);
}
