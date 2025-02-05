use chrono::Utc;
use sha2::{Digest, Sha256};
use url::Url;

pub fn generate_filename(url: &Url) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_str().as_bytes());
    let url_hash = &hex::encode(hasher.finalize())[..8];

    format!("{}-{}", Utc::now().timestamp(), url_hash)
}
