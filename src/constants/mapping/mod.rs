// Autogenerated code doesn't need checking
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use super::*;

#[cfg(feature = "constantgen")]
mod generated {
    use super::*;
    // This file is generated during the build process
    include!(concat!(env!("OUT_DIR"), "/mqconstants.rs"));
}

#[cfg(not(feature = "constantgen"))]
mod generated {
    use super::*;
    // This file is pregenerated

    #[cfg(all(target_os="windows", target_arch="x86_64"))]
    include!("pregen/x86_64-windows-mqconstants.rs");

    #[cfg(all(target_os="linux", target_arch="x86_64"))]
    include!("pregen/x86_64-linux-mqconstants.rs");
}

pub use generated::*;

type MqxaSource<'a> = ConstSource<BinarySearchSource<'a>, BinarySearchSource<'a>>;

/// Selectors for MQIA and MQCA combined
pub const MQXA_FULL_CONST: MqxaSource = ConstSource(mapping::MQIA_CONST, mapping::MQCA_CONST);

/// Return codes can sometimes be in the MQRCCF range so combine the two
pub const MQRC_FULL_CONST: ConstSource<PhfSource, PhfSource> = ConstSource(mapping::MQRC_CONST, mapping::MQRCCF_CONST);
