use chrono::Utc;
use url::Url;

use crate::common::sha256;

pub fn generate_filename(url: &Url) -> String {
    format!("{}-{}", Utc::now().timestamp(), &sha256(url.as_str())[..8])
}
