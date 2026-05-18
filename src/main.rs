pub mod engine;
pub mod neural_network;
use crate::engine::Value;

fn main() {
    let a = Value::init(2.0);
    let b = Value::init(3.0);
    let c = a.powf(2.0) + b.powf(3.0) * 9.0;
    let d = c / 3.0;
    println!("c: {:?}", d.data);
    let z = d.backward();
    print!("{}\n", z.grad);
}
