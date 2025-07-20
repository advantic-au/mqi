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
        $crate::macros::reverse_ident!([], [$($t),*])
    };
    ([$($r:ident),*], [$h:ident $(, $t:ident)*]) => {
        $crate::macros::reverse_ident!([$h $(,$r)*], [$($t),*])
    };
}

macro_rules! impl_option_tuple {
    ($trait:ident, $ty:ty, [$($gen:ident),*] $(,$safety:ident)?) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        $($safety)? impl<$($gen, )*> $trait for ($($gen, )*)
        where
            $($gen: $trait),*
        {
            #[inline]
            fn apply_param(&self, param: &mut $ty) {
                let $crate::macros::reverse_ident!($($gen),*) = self;
                $($gen.apply_param(param);)*
            }
        }
    };

    ($lt:lifetime, $trait:ident, $ty:ty, [$($gen:ident),*] $(,$safety:ident)?) => {
        #[expect(non_snake_case)]
        #[diagnostic::do_not_recommend]
        $($safety)? impl<$lt, $($gen, )*> $trait<$lt> for ($($gen, )*)
        where
            $($gen: $trait<$lt>),*
        {
            #[inline]
            fn apply_param(&self, param: &mut $ty) {
                let crate::macros::reverse_ident!($($gen),*) = self;
                $($gen.apply_param(param);)*
            }
        }
    }
}

macro_rules! all_option_tuples {
    ($trait:ident, $ty:ty $(,$safety:ident)?) => {
        impl<T: $trait> $trait for Option<T> {
            fn apply_param(&self, param: &mut $ty) {
                if let Some(value) = self {
                    value.apply_param(param);
                }
            }
        }
        impl $trait for () {
            fn apply_param(&self, _param: &mut $ty) {}
        }

        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5, M6] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5, M6, M7] $(,$safety)?);
        $crate::macros::impl_option_tuple!($trait, $ty, [M1, M2, M3, M4, M5, M6, M7, M8] $(,$safety)?);
    };
    ($lt:lifetime, $trait:ident, $ty:ty $(,$safety:ident)?) => {
        $($safety)? impl<$lt, T: $trait<$lt>> $trait<$lt> for Option<T> {
            fn apply_param(&self, param: &mut $ty) {
                if let Some(value) = self {
                    value.apply_param(param);
                }
            }
        }
        $($safety)? impl<$lt> $trait<$lt> for () {
            fn apply_param(&self, _param: &mut $ty) {}
        }

        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M6, M7] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5, M6, M7] $(,$safety)?);
        $crate::macros::impl_option_tuple!($lt, $trait, $ty, [M1, M2, M3, M4, M5, M6, M7, M8] $(,$safety)?);
    };
}

/// Delegates `FromStr` to wrapped type implementation
macro_rules! impl_from_str {
    ($i:ident, $ty:ty) => {
        impl std::str::FromStr for $i {
            type Err = <$ty as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$ty as std::str::FromStr>::from_str(s)?))
            }
        }
    };
}

pub(crate) use all_multi_tuples;
pub(crate) use all_option_tuples;
pub(crate) use impl_from_str;
pub(crate) use impl_option_tuple;
pub(crate) use reverse_ident;
