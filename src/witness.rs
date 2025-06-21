use ff::Field;


pub trait Witness<F: Field> {
    #[proc_macro]
    fn into_witness(self) -> Vec<F>;
}