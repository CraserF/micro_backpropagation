pub mod engine;
pub mod neural_network;
use crate::engine::Value;

fn main() {
    let m = Value::init(2.0);
    let n = Value::init(3.0);
    let mn = m.powf(2.0) + n.powf(3.0) * 9.0;
    let zn = mn / 3.0;
    println!("zn: {:?}", zn.data);
    let yz = zn.backward();
    print!("yz: {}\n", yz.grad);

    let a = Value::init(-4.0);
    let b = Value::init(2.0);
    let c = a.clone() + b.clone(); // -2
    let d = a.clone() * b.clone() + b.clone().powf(3.0); // 0
    print!("d: {}\n", d.data);
    let c = c.clone() + c.clone() + 1.0; // -2
    let c = (c.clone() + 1.0) + c.clone() + (-a.clone());
    println!("c: {}", c.data);
    let d = d.clone() + d.clone() * 2.0 + (b.clone() + a.clone()).relu();
    println!("d: {}", d.data);
    let d = d.clone() + d.clone() * 3.0 + (b.clone() - a.clone()).relu();
    println!("d: {}", d.data);
    let e = c - d;
    println!("e: {}", e.data);
    let f = e.powf(2.0);
    println!("f: {}", f.data);
    let g = f.clone() / 2.0;
    let g = g.clone() + 10.0 / f;
    print!("Data: {:.4} \n", g.data); // prints 24.7041, the outcome of this forward pass
    g.backward();
}
