use password_auth::{generate_hash, verify_password};

pub fn hash(password: &str) -> String {
    generate_hash(password)
}

pub fn verify(password: &str, hashed: &str) -> bool {
    verify_password(password, hashed).is_ok()
}
