// Concept adapted from the axum crate
// Used to implement a macro for tuples
macro_rules! all_multi_tuples {
    ($name:ident) => {
        $name!(M1, [M2]);
        $name!(M1, [M2, M3]);
        $name!(M1, [M2, M3, M4]);
        $name!(M1, [M2, M3, M4, M5]);
        $name!(M1, [M2, M3, M4, M5, M6]);
        $name!(M1, [M2, M3, M4, M5, M6, M7]);
        $name!(M1, [M2, M3, M4, M5, M6, M7, M8]);
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

pub(crate) use all_multi_tuples;
pub(crate) use reverse_ident;
