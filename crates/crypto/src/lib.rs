pub trait CodeGenerator: Send + Sync + 'static {
    fn generate(&self, len: usize) -> Vec<u8>;
}

use rand::Rng as _;

#[derive(Default)]
pub struct OtpCodeGenerator {}

impl CodeGenerator for OtpCodeGenerator {
    fn generate(&self, len: usize) -> Vec<u8> {
        let max_value = 10_u32.pow(len as u32);

        let mut rng = rand::rng();
        let num = rng.random_range(0..max_value);

        format!("{num:len$}").into_bytes()
    }
}
