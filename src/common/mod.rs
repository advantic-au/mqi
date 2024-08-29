pub mod macros;
mod mqmask;
mod mqstr;
mod mqvalue;
mod parameters;
mod result;

pub use mqmask::*;
pub use mqstr::*;
pub use mqvalue::*;
pub use result::*;
pub(super) use parameters::*;

#[macro_export]
macro_rules! define_mqvalue {
    ($vis:vis $i:ident, $source:path) => {
        #[derive(Copy, Clone)]
		$vis struct $i;
		$crate::impl_constant_lookup!($i, $source);
    };
}

#[macro_export]
macro_rules! impl_default_mqvalue {
    ($t:path, $source:path) => {
        impl Default for $t {
            fn default() -> Self {
                Self::from($source)
            }
        }
    };
}
