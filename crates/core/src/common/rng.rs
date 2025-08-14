use rand::Rng as _;

pub struct NumericCodeGenerator;

impl NumericCodeGenerator {
    pub fn generate(len: u8) -> Vec<u8> {
        let max_value = 10_u32.pow(len as u32);

        let mut rng = rand::rng();
        let num = rng.random_range(0..max_value);

        format!("{:0width$}", num, width = len as usize).into_bytes()
    }
}
