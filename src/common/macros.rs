// Concept adapted from the axum crate
// Used to implement a macro for tuples
macro_rules! all_multi_tuples {
    ($name:ident) => {
        $name!([M1, M2]);
        $name!([M1, M2, M3]);
        $name!([M1, M2, M3, M4]);
        $name!([M1, M2, M3, M4, M5]);
        $name!([M1, M2, M3, M4, M5, M6]);
        $name!([M1, M2, M3, M4, M5, M6, M7]);
        $name!([M1, M2, M3, M4, M5, M6, M7, M8]);
    };
}

// Nasty little macro to reverse a list of idents
macro_rules! reverse_ident {
    ([$($r:ident),*], []) => {
        ($($r),*)
    };
    ($($t:ident),*) => {
        reverse_ident!([], [$($t),*])
    };
    ([$($r:ident),*], [$h:ident $(, $t:ident)*]) => {
        reverse_ident!([$h $(,$r)*], [$($t),*])
    };
}

macro_rules! impl_option_tuple {
    ($trait:ident, $ty:ty, [$first:ident, $($gen:ident),*]) => {
        #[expect(non_snake_case)]
        impl<$first, $($gen, )*> $trait for ($first, $($gen, )*)
        where
            $first: $trait,
            $($gen: $trait),*
        {
            #[inline]
            fn apply_param(self, param: &mut $ty) {
                let ($first, $($gen, )*) = self;
                ($($gen),*).apply_param(param);
                $first.apply_param(param);
            }
        }
    };

    ($lt:lifetime, $trait:ident, $ty:ty, [$first:ident, $($gen:ident),*]) => {
        #[expect(non_snake_case)]
        impl<$lt, $first, $($gen, )*> $trait<$lt> for ($first, $($gen, )*)
        where
            $first: $trait<$lt>,
            $($gen: $trait<$lt>),*
        {
            #[inline]
            fn apply_param(self, param: &mut $ty) {
                let ($first, $($gen, )*) = self;
                ($($gen),*).apply_param(param);
                $first.apply_param(param);
            }
        }
    }
}

macro_rules! all_option_tuples {
    ($trait:ident, $ty:ty) => {
        impl<T: $trait> $trait for Option<T> {
            fn apply_param(self, param: &mut $ty) {
                if let Some(value) = self {
                    value.apply_param(param);
                }
            }
        }
        impl $trait for () {
            fn apply_param(self, _param: &mut $ty) {}
        }

        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M6, M7]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5, M6, M7]);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5, M6, M7, M8]);
    };
    ($lt:lifetime, $trait:ident, $ty:ty) => {
        impl<$lt, T: $trait<$lt>> $trait<$lt> for Option<T> {
            fn apply_param(self, param: &mut $ty) {
                if let Some(value) = self {
                    value.apply_param(param);
                }
            }
        }
        impl<$lt> $trait<$lt> for () {
            fn apply_param(self, _param: &mut $ty) {}
        }

        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M6, M7]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5, M6, M7]);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5, M6, M7, M8]);
    };
}

pub(crate) use impl_option_tuple;
pub(crate) use all_option_tuples;
pub(crate) use all_multi_tuples;
pub(crate) use reverse_ident;
