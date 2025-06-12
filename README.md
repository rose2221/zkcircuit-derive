# zkcircuit-derive: Boilerplate-Free ZK Circuits in Rust

[![Crates.io](https://img.shields.io/crates/v/zkcircuit-derive.svg)](https://crates.io/crates/zkcircuit-derive)
[![Build Status](https://img.shields.io/travis/your-username/zkcircuit-derive.svg)](https://travis-ci.org/your-username/zkcircuit-derive)

`zkcircuit-derive` is a procedural macro that automates the tedious and error-prone process of writing boilerplate for ZK-SNARK circuits using the `bellman` framework. Simply define the logic of your circuit in a plain Rust struct, and let the macro handle the rest.

---

## The Problem

Building Zero-Knowledge circuits involves manually wiring up every input and witness variable to the constraint system. This process is repetitive and a common source of bugs. For a simple circuit that proves `a + b = c`, you have to write code to allocate `a`, allocate `b`, allocate `c`, and then enforce the constraint.

This crate solves that problem by letting you describe the *what* (the data in your circuit) and automatically generating the *how* (the `Circuit` trait implementation).

## Features

- **Automatic `Circuit` Implementation:** Generates a full `bellman::Circuit::synthesize` implementation from a struct definition.
- **Input vs. Witness:** Differentiates between public inputs (`#[circuit(input)]`) and private witness values.
- **Ergonomic Builder Pattern:** Automatically generates a companion `Builder` struct for safely and easily constructing your circuit instance.
- **Handles Generics:** Works seamlessly with generic circuit structs (e.g., `MyCircuit<F: ff::Field>`).

---

## Quick Start

1.  Add `zkcircuit-derive` and `bellman` to your `Cargo.toml`:

    ```toml
    [dependencies]
    bellman = "0.10" # Or your desired version
    ff = "0.12"
    zkcircuit-derive = "0.1.0"
    ```

2.  Define your circuit's data as a struct. All fields must be `Option<F>`. Use the `#[derive(zkcircuit)]` macro and annotate public inputs.

    ```rust
    use ff::Field;
    use zkcircuit_derive::zkcircuit;

    #[derive(zkcircuit)]
    struct ExampleCircuit<F: Field> {
        // Public inputs are marked with an attribute
        #[circuit(input)]
        pub a: Option<F>,

        // Private witness values have no attribute
        pub b: Option<F>,
    }
    ```

3.  Use the auto-generated builder to create an instance of your circuit.

    ```rust
    // In a test or your main application...
    // let builder = ExampleCircuitBuilder::new();
    // let circuit_instance = builder
    //     .a(some_field_element)
    //     .b(another_field_element)
    //     .build();
    ```

---

## What the Macro Generates

The `#[derive(zkcircuit)]` macro expands your simple struct into the following code:

1.  **A `Circuit` implementation** that correctly allocates each field as either a public input or private witness.

    ```rust
    // Generated Code (Simplified)
    impl<F: ff::Field> bellman::Circuit<F> for ExampleCircuit<F> {
        fn synthesize<CS: bellman::ConstraintSystem<F>>(
            self,
            cs: &mut CS,
        ) -> Result<(), bellman::SynthesisError> {
            // Allocates `a` as a public input
            let a = cs.alloc_input(
                &mut cs.namespace(|| "a"),
                || self.a.ok_or(bellman::SynthesisError::AssignmentMissing),
            )?;

            // Allocates `b` as a private witness
            let b = cs.alloc(
                &mut cs.namespace(|| "b"),
                || self.b.ok_or(bellman::SynthesisError::AssignmentMissing),
            )?;

            // (You would add your constraint logic here manually for now)

            Ok(())
        }
    }
    ```

2.  **A `Builder` struct** with a fluent API to ensure all fields are provided.

    ```rust
    // Generated Code
    pub struct ExampleCircuitBuilder<F: ff::Field> {
        pub a: Option<F>,
        pub b: Option<F>,
    }

    impl<F: ff::Field> ExampleCircuitBuilder<F> {
        pub fn new() -> Self { /* ... */ }

        pub fn a(mut self, value: F) -> Self {
            self.a = Some(value);
            self
        }

        pub fn b(mut self, value: F) -> Self {
            self.b = Some(value);
            self
        }

        pub fn build(self) -> ExampleCircuit<F> {
            ExampleCircuit {
                a: self.a,
                b: self.b,
            }
        }
    }
    ```

## License

This project is licensed under the MIT License.
