#[derive(Debug)]
pub struct RegisterDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}
