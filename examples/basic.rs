
use zkcircuit_derive::witness::Witness;
use ff::Field;
use bellman::{
    groth16::{create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof},
    Circuit, ConstraintSystem, SynthesisError,
};
use pairing::bls12_381::{Bls12, Fr}; // Using the BLS12-381 curve
use rand::thread_rng;

#[derive(zkcircuit_derive::zkcircuit, Clone)]
    struct Cubedemo {
        pub x : Option<F>
    }

    impl<F: Field> Circuit<F> for Cubedemo<F> {
        fn synthesize<CS: ConstraintSystem<F>>(self, cs: &mut CS) -> Result<(), SynthesisError>{
            let x_var = cs.alloc(||"x" , || self.x.ok_or(SynthesisError
            ::AssignmentMissing))?;
            let x_squared = cs.alloc(||"x_squared", || {
                let mut tmp = self.x.ok_or(SynthesisError::AssignmentMissing)?;
                tmp.square();
                Ok(tmp)
            } )?;
        }
    }