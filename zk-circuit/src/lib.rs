use ff::Field;


pub trait Witness<F: Field> {
   
    fn into_witness(self) -> Vec<F>;
}