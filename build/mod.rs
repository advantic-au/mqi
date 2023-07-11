#[cfg(feature = "constantgen")]
mod constants {
    mod list;
    pub mod generate;
}

fn main() {
    #[cfg(feature = "constantgen")]
    constants::generate::generate();
}
