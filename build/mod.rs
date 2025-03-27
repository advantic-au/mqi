use std::{env, io};

#[cfg(feature = "constantgen")]
mod constants {
    pub mod generate;
    mod list;
}

#[expect(clippy::allow_attributes)]
#[allow(clippy::unnecessary_wraps, reason = "when no features are enabled")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(unused_variables, reason = "when constantgen and pregen are not enabled")]
    let path = std::path::Path::new(&env::var("OUT_DIR").expect("OUT_DIR is mandatory for builds")).join("mqconstants.rs");
    #[cfg(feature = "constantgen")]
    {
        use std::io::Write as _;
        let mut constants_write = io::BufWriter::new(Vec::new());
        constants::generate::generate_constants(&mut constants_write)?;

        let constants_str = String::from_utf8(constants_write.into_inner()?)?;
        let constants_syn = syn::parse_file(&constants_str)?;
        let constants_pretty = prettyplease::unparse(&constants_syn);
        let constants_file = std::fs::File::create(&path)?;
        let mut constants_pretty_write = io::BufWriter::new(constants_file);

        writeln!(
            &mut constants_pretty_write,
            "/* Generated with MQ client version {} */",
            libmqm_sys::version::CLIENT_BUILD_VERSION
        )?;
        constants_pretty_write.write_all(constants_pretty.as_bytes())?;
    }

    #[cfg(feature = "pregen")]
    {
        use std::{env::consts as env_consts, fs, path};

        fs::copy(
            &path,
            path::PathBuf::from("./src/constants/mapping/pregen").join(format!(
                "{}-{}-mqconstants.rs",
                if env_consts::OS == "macos" { "any" } else { env_consts::ARCH },
                env_consts::OS
            )),
        )?;
    }

    Ok(())
}
