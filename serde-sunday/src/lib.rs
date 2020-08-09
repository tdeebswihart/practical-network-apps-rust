use rand::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
}

impl Move {
    pub fn new(x: i32, y: i32) -> Self {
        Move { x, y }
    }

    pub fn random() -> Self {
        Move {
            x: random(),
            y: random(),
        }
    }
}
