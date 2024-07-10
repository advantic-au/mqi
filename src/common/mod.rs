mod mqmask;
mod mqstr;
mod mqvalue;
mod result;

pub use mqmask::*;
pub use mqstr::*;
pub use mqvalue::*;
pub use result::*;

#[macro_export]
macro_rules! define_mqvalue {
    ($vis:vis $i:ident, $source:path) => {
        #[derive(Copy, Clone)]
		$vis struct $i;
		$crate::impl_constant_lookup!($i, $source);
    };
}
