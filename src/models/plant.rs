use crate::bonsai;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum PlantType {
    Bonsai,
}

impl Default for PlantType {
    fn default() -> Self {
        PlantType::Bonsai
    }
}

pub struct Plant {
    pub seed: u64,
    pub plant_type: PlantType,
    pub progress: f32,
}

impl Plant {
    pub fn new(seed: u64, plant_type: PlantType, progress: f32) -> Self {
        Self { seed, plant_type, progress }
    }
}

pub fn generate_plant(plant: &Plant) -> bonsai::canvas::BonsaiCanvas {
    match plant.plant_type {
        PlantType::Bonsai => bonsai::generate_bonsai(plant.seed, plant.progress),
    }
}
