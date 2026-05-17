# micro_backpropagation

`micro_backpropagation` is a hand-coded Rust learning project inspired by Andrej Karpathy's [Micrograd](https://github.com/karpathy/micrograd), with a Rust spin.

The goal is to make backpropagation understandable by building the core pieces from scratch: scalar values, operation tracking, and gradient propagation through a small computation graph.

## Why this project exists

- Learn and study backpropagation mechanics in a minimal codebase
- Recreate Micrograd-style ideas in idiomatic Rust where possible
- Keep implementation simple and inspectable rather than production-focused

## Current structure

- `src/engine.rs` - core `Value` type and operator wiring (`Add`, `Mul`, `Div`, `Sub`, `powf`)
- `src/neural_network.rs` - early neural-network-layer experiments
- `src/main.rs` - crate entry point and module declarations

## Run

```bash
cargo run
```

## Notes

This is an educational, work-in-progress implementation intended for studying and explaining autograd/backpropagation concepts.
