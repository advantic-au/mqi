pub mod macros;
pub mod mqmask;
mod mqstr;
pub mod mqvalue;
mod result;

pub use mqstr::*;
pub use result::*;

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
