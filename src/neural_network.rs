use crate::engine::Value;
use rand::Rng;

pub struct Neuron {
    pub weight: Value,
    pub bias: Value,
    pub is_non_linear: bool,
}

// impl Neuron {
//     pub fn init (numberOfInputs: u32) -> Self {
//         Neuron { weight: rand::thread_rng().gen() , bias: Value(0), isNonlinear: () }
//     }
// }
