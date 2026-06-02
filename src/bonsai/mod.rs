pub mod canvas;
pub mod generator;
pub mod cactus_generator;
pub mod bush_generator;
pub mod scale;

pub use canvas::BonsaiCell;
pub use generator::generate_bonsai;
pub use cactus_generator::generate_cactus;
pub use bush_generator::generate_bush;
pub use scale::{PlantFrame, Scale, SCALES};
