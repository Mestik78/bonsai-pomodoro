use crate::bonsai;
use serde::{Deserialize, Serialize};
use ratatui::style::Color;

#[derive(Clone, PartialEq)]
pub struct Fruit {
    pub character: char,
    pub color: Color,
}

impl Fruit {
    pub fn new(character: char, color: Color) -> Self {
        Self { character, color }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum PlantType {
    // Bonsai derivatives
    Bonsai,
    Sakura,
    LemonTree,
    AppleTree,
    OrangeTree,
    // Oak derivatives
    Oak,
    RedMaple,
    WhiteBirch,
    // Mushroom derivatives
    Mushroom,
    TallMushroom,
    BrownMushroom,
    // Unique plants
    Pine,
    Bamboo,
    Willow,
    Cactus,
    Bush,
    // Fallback
    #[serde(untagged)]
    Unknown(String),
}

impl Default for PlantType {
    fn default() -> Self {
        PlantType::Bonsai
    }
}

impl PlantType {
    pub fn all() -> Vec<PlantType> {
        vec![
            PlantType::Bonsai, PlantType::Sakura, PlantType::LemonTree, PlantType::AppleTree, PlantType::OrangeTree,
            PlantType::Oak, PlantType::RedMaple, PlantType::WhiteBirch,
            PlantType::Mushroom, PlantType::TallMushroom, PlantType::BrownMushroom,
            PlantType::Pine, PlantType::Bamboo, PlantType::Willow, PlantType::Cactus, PlantType::Bush,
        ]
    }
    
    pub fn to_string(&self) -> String {
        match self {
            PlantType::Bonsai => "Bonsai".to_string(),
            PlantType::Sakura => "Sakura".to_string(),
            PlantType::LemonTree => "Lemon Tree".to_string(),
            PlantType::AppleTree => "Apple Tree".to_string(),
            PlantType::OrangeTree => "Orange Tree".to_string(),
            PlantType::Oak => "Oak Tree".to_string(),
            PlantType::RedMaple => "Red Maple".to_string(),
            PlantType::WhiteBirch => "White Birch".to_string(),
            PlantType::Mushroom => "Red Mushroom".to_string(),
            PlantType::TallMushroom => "Tall Mushroom".to_string(),
            PlantType::BrownMushroom => "Brown Mushroom".to_string(),
            PlantType::Pine => "Pine Tree".to_string(),
            PlantType::Bamboo => "Bamboo".to_string(),
            PlantType::Willow => "Weeping Willow".to_string(),
            PlantType::Cactus => "Cactus".to_string(),
            PlantType::Bush => "Bush".to_string(),
            PlantType::Unknown(s) => s.clone(),
        }
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

    pub fn from_timer(timer: &crate::models::timer::TimerSession) -> Self {
        Self {
            seed: timer.seed,
            plant_type: timer.plant_type.clone(),
            progress: timer.progress(),
        }
    }
}

pub fn generate_plant(plant: &Plant) -> bonsai::canvas::BonsaiCanvas {
    match plant.plant_type {
        PlantType::Bonsai => bonsai::generate_bonsai(plant.seed, plant.progress, None, 0),
        PlantType::Sakura => {
            let blossom = Fruit::new('*', Color::Rgb(255, 183, 197));
            bonsai::generate_bonsai(plant.seed, plant.progress, Some(blossom), 80)
        },
        PlantType::LemonTree => {
            let lemon = Fruit::new('●', Color::Rgb(255, 244, 79));
            bonsai::generate_bonsai(plant.seed, plant.progress, Some(lemon), 4)
        },
        PlantType::AppleTree => {
            let apple = Fruit::new('●', Color::Rgb(220, 20, 60)); // Crimson Red
            bonsai::generate_bonsai(plant.seed, plant.progress, Some(apple), 4)
        },
        PlantType::OrangeTree => {
            let orange = Fruit::new('●', Color::Rgb(255, 165, 0)); // Orange
            bonsai::generate_bonsai(plant.seed, plant.progress, Some(orange), 4)
        },
        PlantType::Oak => bonsai::generate_oak(plant.seed, plant.progress, None, 0),
        PlantType::RedMaple => bonsai::generate_red_maple(plant.seed, plant.progress, None, 0),
        PlantType::WhiteBirch => bonsai::generate_white_birch(plant.seed, plant.progress, None, 0),
        PlantType::Mushroom => bonsai::generate_mushroom(plant.seed, plant.progress, None, 0),
        PlantType::TallMushroom => bonsai::generate_tall_mushroom(plant.seed, plant.progress, None, 0),
        PlantType::BrownMushroom => bonsai::generate_brown_mushroom(plant.seed, plant.progress, None, 0),
        PlantType::Pine => bonsai::generate_pine(plant.seed, plant.progress, None, 0),
        PlantType::Bamboo => bonsai::generate_bamboo(plant.seed, plant.progress, None, 0),
        PlantType::Willow => bonsai::generate_willow(plant.seed, plant.progress, None, 0),
        PlantType::Cactus => {
            let flower = Fruit::new('✿', Color::Rgb(255, 20, 147));
            bonsai::generate_cactus(plant.seed, plant.progress, Some(flower), 2)
        },
        PlantType::Bush => {
            bonsai::generate_bush(plant.seed, plant.progress, None, 0)
        },
        PlantType::Unknown(_) => bonsai::generate_bonsai(plant.seed, plant.progress, None, 0),
    }
}
