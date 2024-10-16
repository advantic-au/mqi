use std::{env, io, path::Path};

#[cfg(feature = "constantgen")]
mod constants {
    pub mod generate;
    mod list;
}

fn main() -> Result<(), io::Error> {
    let path = Path::new(&env::var("OUT_DIR").expect("OUT_DIR is mandatory for builds")).join("mqconstants.rs");
    #[cfg(feature = "constantgen")]
    constants::generate::generate(&path);

    #[cfg(feature = "pregen")]
    {
        use std::{env::consts as env_consts, fs, path};

        fs::copy(
            &path,
            path::PathBuf::from("./src/constants/mapping/pregen").join(format!(
                "{}-{}-mqconstants.rs",
                if env_consts::OS == "macOS" { "any" } else { env_consts::ARCH },
                env_consts::OS
            )),
        )?;
    }

    Ok(())
}
