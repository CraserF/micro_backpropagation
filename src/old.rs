use std::ops::{Add, Mul, Sub, Div};

// we can also use a static vector that all of them reference

#[derive(Clone, Copy)]
enum BinaryOp { Add, Mul, Sub, Div }
 
#[derive(Clone, Copy)]
enum FlatOp {
    Binary(BinaryOp, usize, usize), // op, left_idx, right_idx in vec
    Power(usize, f64),
}
 
pub struct FlatNode {
    pub data: f64,
    pub grad: f64,
    op: Option<FlatOp>,
}
 
pub struct Value {
    pub data: f64,
    previous: Option<Box<ValueOperation>>,
}
 
enum ValueOperation {
    Addition(Value, Value),
    Multiplication(Value, Value),
    Subtraction(Value, Value),
    Division(Value, Value),
    Power(Value, f64),
}
 
impl Value {
    pub fn new(data: f64) -> Value {
        Value { data, previous: None }
    }
 
    pub fn powf(self, exp: f64) -> Value {
        Value {
            data: self.data.powf(exp),
            previous: Some(Box::new(ValueOperation::Power(self, exp))),
        }
    }
 
    /// Consumes the Value tree, flattens into topo-sorted Vec, computes gradients.
    pub fn backward(self) -> Vec<FlatNode> {
        let mut nodes: Vec<FlatNode> = Vec::new();
 
        // Recursively consume the tree, returning each node's index
        fn flatten(value: Value, nodes: &mut Vec<FlatNode>) -> usize {
            let op = match value.previous {
                None => None,
                Some(boxed) => match *boxed {
                    ValueOperation::Addition(l, r) => {
                        let li = flatten(l, nodes);
                        let ri = flatten(r, nodes);
                        Some(FlatOp::Binary(BinaryOp::Add, li, ri))
                    }
                    ValueOperation::Multiplication(l, r) => {
                        let li = flatten(l, nodes);
                        let ri = flatten(r, nodes);
                        Some(FlatOp::Binary(BinaryOp::Mul, li, ri))
                    }
                    ValueOperation::Subtraction(l, r) => {
                        let li = flatten(l, nodes);
                        let ri = flatten(r, nodes);
                        Some(FlatOp::Binary(BinaryOp::Sub, li, ri))
                    }
                    ValueOperation::Division(l, r) => {
                        let li = flatten(l, nodes);
                        let ri = flatten(r, nodes);
                        Some(FlatOp::Binary(BinaryOp::Div, li, ri))
                    }
                    ValueOperation::Power(base, exp) => {
                        let bi = flatten(base, nodes);
                        Some(FlatOp::Power(bi, exp))
                    }
                },
            };
            let idx = nodes.len();
            nodes.push(FlatNode { data: value.data, grad: 0.0, op });
            idx
        }
 
        let root = flatten(self, &mut nodes);
        nodes[root].grad = 1.0; // seed gradient at root
 
        // Walk reverse topo order (root is last), propagate chain rule
        for i in (0..nodes.len()).rev() {
            let grad = nodes[i].grad;
            if let Some(op) = nodes[i].op {
                match op {
                    FlatOp::Binary(BinaryOp::Add, l, r) => {
                        nodes[l].grad += grad;
                        nodes[r].grad += grad;
                    }
                    FlatOp::Binary(BinaryOp::Mul, l, r) => {
                        let (ld, rd) = (nodes[l].data, nodes[r].data);
                        nodes[l].grad += grad * rd;
                        nodes[r].grad += grad * ld;
                    }
                    FlatOp::Binary(BinaryOp::Sub, l, r) => {
                        nodes[l].grad += grad;
                        nodes[r].grad -= grad;
                    }
                    FlatOp::Binary(BinaryOp::Div, l, r) => {
                        let (ld, rd) = (nodes[l].data, nodes[r].data);
                        nodes[l].grad += grad / rd;
                        nodes[r].grad -= grad * ld / (rd * rd);
                    }
                    FlatOp::Power(base, exp) => {
                        let bd = nodes[base].data;
                        nodes[base].grad += grad * exp * bd.powf(exp - 1.0);
                    }
                }
            }
        }
 
        nodes // caller can inspect .grad on any node by index
    }
}
