pub mod macros;
mod mqmask;
mod mqstr;
mod mqvalue;
mod parameters;
mod result;

pub use mqstr::*;
pub use result::*;
pub use parameters::*;

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
