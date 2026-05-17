use std::ops::{Add, Div, Mul, Sub};

// Array [Value] => [Grad]
// We need a recursive type that can track how we got here which stores operators such as Added & Multiplied.
// Backward (Value[]) => (Value, Grad)[]
// X = A * B
// Z = X + Y
// L = tanZ
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub data: f64,
    pub grad: f64,
    previous: Option<Box<Previous>>,
}
#[derive(Debug, PartialEq)]
pub enum Previous {
    Add(Value, Value),
    Mul(Value, Value),
    Pow(Value, f64),
    Div(Value, Value),
    Sub(Value, Value),
}

impl Clone for Previous {
    fn clone(&self) -> Self {
        match self {
            Previous::Pow(a, b) => Previous::Pow(a.clone(), b.clone()),
            Previous::Add(a, b)
            | Previous::Div(a, b)
            | Previous::Mul(a, b)
            | Previous::Sub(a, b) => Previous::Add(a.clone(), b.clone()),
        }
    }
}

impl Value {
    pub fn clone(&self) -> Self {
        Value {
            data: self.data,
            grad: self.grad,
            previous: self.previous.clone(),
        }
    }
    // Initializes a new Value with the given data.
    pub fn init(data: f64) -> Self {
        Value {
            data,
            grad: 0.0,
            previous: None,
        }
    }
    pub fn powf(self, other: f64) -> Self {
        Value {
            data: self.data.powf(other),
            grad: 0.0,
            previous: Some(Box::new(Previous::Pow(self, other))),
        }
    }
    // Addition: z = a + b → a.grad += z.grad * 1.0; b.grad += z.grad * 1.0
    // Multiplication: z = a * b → a.grad += z.grad * b.data; b.grad += z.grad * a.data
    // Subtraction: z = a - b → a.grad += z.grad * 1.0; b.grad += z.grad * -1.0
    // Division: z = a / b → a.grad += z.grad * (1.0 / b.data) b.grad += z.grad * (-a.data / b.data²)
    // Power:z = a^n → a.grad += z.grad * (n * a.data^(n-1))
    pub fn backward(self) {
        let mut topo: Vec<Value> = vec![];
        fn build_topo(value: Value, topo: &mut Vec<Value>) {
            if !topo.contains(&value) {
                match value.previous {
                    None => {}
                    Some(previous) => match *previous {
                        Previous::Pow(a, _) => build_topo(a, topo),
                        Previous::Add(a, b)
                        | Previous::Mul(a, b)
                        | Previous::Div(a, b)
                        | Previous::Sub(a, b) => {
                            build_topo(a, topo);
                            build_topo(b, topo);
                        }
                    },
                }
            }
        }
        build_topo(self, &mut topo);
        let mut values: Vec<Value> = vec![];
        for value in topo {
            match value.previous {
                None => {}
                Some(previous) => {
                    values.push(value);

                    let value = *previous;
                    match value {
                        Previous::Add(lhs, rhs) => {
                            lhs.grad += self.grad + rhs.grad;
                        }
                        Previous::Sub(lhs, rhs) => {}
                        Previous::Mul(lhs, rhs) => {}
                        Previous::Pow(lhs, rhs) => {}
                        Previous::Div(lhs, rhs) => {}
                    }
                }
            }
        }
    }
}

impl Mul<Value> for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Value {
            data: self.data * rhs.data,
            grad: 0.0,
            previous: Some(Box::new(Previous::Mul(self, rhs))),
        }
    }
}

impl Mul<f64> for Value {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        Value {
            data: self.data * rhs,
            grad: 0.0,
            previous: Some(Box::new(Previous::Mul(self, Value::init(rhs)))),
        }
    }
}

impl Add<Value> for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        Value {
            data: self.data + rhs.data,
            grad: 0.0,
            previous: Some(Box::new(Previous::Add(self, rhs))),
        }
    }
}

impl Add<Value> for f64 {
    type Output = Value;
    fn add(self, rhs: Value) -> Value {
        rhs + self
    }
}

impl Add<f64> for Value {
    type Output = Value;
    fn add(self, rhs: f64) -> Value {
        Value {
            data: self.data + rhs,
            previous: Some(Box::new(Previous::Add(self, Value::init(rhs)))),
            grad: 0.0,
        }
    }
}

impl Div<Value> for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self * rhs.powf(-1.0)
    }
}

impl Div<f64> for Value {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        self * rhs.powi(-1)
    }
}

impl Sub<Value> for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (rhs * -1.0)
    }
}
