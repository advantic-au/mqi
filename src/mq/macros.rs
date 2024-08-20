macro_rules! all_multi_tuples {
    ($name:ident) => {
        $name!(T1, [T2]);
        $name!(T1, [T2, T3]);
        $name!(T1, [T2, T3, T4]);
        $name!(T1, [T2, T3, T4, T5]);
        $name!(T1, [T2, T3, T4, T5, T6]);
        $name!(T1, [T2, T3, T4, T5, T6, T7]);
        $name!(T1, [T2, T3, T4, T5, T6, T7, T8]);
    };
}

pub(crate) use all_multi_tuples;