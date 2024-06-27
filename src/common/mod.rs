mod mqstr;
mod mqvalue;
mod mqmask;
mod result;

pub use mqstr::*;
pub use mqvalue::*;
pub use mqmask::*;
pub use result::*;


#[macro_export]
macro_rules! define_mqvalue {
    ($i:ident, $source:path) => {
        #[derive(Copy, Clone)]
		pub struct $i;
		$crate::impl_constant_lookup!($i, $source);
    };
}
