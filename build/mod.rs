#[cfg(feature = "constantgen")]
mod constants {
    pub mod generate;
    mod list;
}

fn main() {
    #[cfg(feature = "constantgen")]
    constants::generate::generate();
}
