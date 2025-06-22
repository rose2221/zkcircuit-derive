use bellman::{
    groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    },
    Circuit, ConstraintSystem, SynthesisError,
};

use bellman::gadgets::test::TestConstraintSystem;
use bls12_381::{Bls12, Scalar as Fr}; // Using the BLS12-381 curve
use ff::Field;
use ff::PrimeField;
use pairing::Engine;
use rand::rngs::OsRng;
use zkcircuit::Witness;
use zkcircuit_derive::zkcircuit;
// use crate::Witness;
use std::str::FromStr;

#[derive(zkcircuit, Clone)]
struct CubeDemo<F: PrimeField> {
    pub x: Option<F>,
}

struct CubeDemoCircuit<F: PrimeField>(CubeDemo<F>);

impl<F: PrimeField> Circuit<F> for CubeDemoCircuit<F> {
    fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let x_val = self.0.x;
        let x_var = cs.alloc(|| "x", || x_val.ok_or(SynthesisError::AssignmentMissing))?;
        let x_squared = cs.alloc(
            || "x_squared",
            || {
                let mut tmp = x_val.ok_or(SynthesisError::AssignmentMissing)?;
                tmp.square();
                Ok(tmp)
            },
        )?;
        cs.enforce(
            || "x_squared = x*x",
            |lc| lc + x_var,
            |lc| lc + x_var,
            |lc| lc + x_squared,
        );
        let x_cubed = cs.alloc(
            || "x_cubed",
            || {
                let mut tmp = x_val.ok_or(SynthesisError::AssignmentMissing)?;
                tmp.square();
                tmp.mul_assign(&x_val.ok_or(SynthesisError::AssignmentMissing)?);
                Ok(tmp)
            },
        )?;
        cs.enforce(
            || "x_squared * x = x_cubed",
            |lc| lc + x_squared, // Part A is x_squared
            |lc| lc + x_var,         // Part B is x
            |lc| lc + x_cubed,    // Part C is x_cubed
        );
        let public_input = F::from_str_vartime("35").unwrap();
        let public_input_var = cs.alloc_input(|| "public_input", || Ok(public_input))?;
        cs.enforce(
            || "x_cubed + x +5 = 35",
            |lc| lc + x_cubed + x_var,
            |lc| lc + CS::one(),
            |lc| lc + public_input_var - (F::from(5), CS::one()),
        );
        Ok(())
    }
}
fn main() {
    println!("starting of zk-snark proof generation x^3 + x+ 5 = 35");
    let mut rng = OsRng;




   


    let params = {
        // let empty_circuit = CubeDemo::<Fr>{ x : None};

        let empty_circuit = CubeDemoCircuit(CubeDemo::<Fr> {
            // any witness that satisfies  x³ + x + 5 = 35  is fine;  x = 3 is simplest
            x: Some(Fr::from(3u64)),
        });

        generate_random_parameters::<Bls12, _, _>(empty_circuit, &mut rng).unwrap()
    };
    let pvk = prepare_verifying_key(&params.vk);
    let secret_x = Fr::from(3u64);
    let circuit = CubeDemoBuilder::new()
        .x(secret_x)
        .build()
        .expect("build should succeeed");



 // --- ADDED: Debugging Section ---
    println!("\n--- Running Debugger ---");
    // Create a new TestConstraintSystem
    let mut cs = TestConstraintSystem::<Fr>::new();
    // Synthesize the circuit with our witness against the test system
    circuit.clone().synthesize(&mut cs).unwrap();

    // Check if the constraints are satisfied
    if cs.is_satisfied() {
        println!("Constraints are satisfied! The logic is correct.");
    } else {
        // If not, find out which constraint is failing. `which_is_unsatisfied` will
        // return the name of the first constraint that fails.
        println!("ERROR: Constraints are NOT satisfied!");
        if let Some(failing_constraint) = cs.which_is_unsatisfied() {
            println!("The failing constraint is: '{}'", failing_constraint);
        }
    }
    println!("----------------------\n");




    let proof = create_random_proof(CubeDemoCircuit(circuit.clone()), &params, &mut rng)
        .expect("proof generation");
    println!("verifying the proof");
    let public_inputs = [Fr::from(35u64)];
    let verification_result = verify_proof(&pvk, &proof, &public_inputs[..]);

    assert!(verification_result.is_ok(), "Proof verification failed");
    println!("Proof verification succeeded");
    let witness = circuit.into_witness();
    println!("Generated witness: {:?}", witness);
    assert_eq!(witness[0], secret_x);
}
