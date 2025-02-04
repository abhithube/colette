use std::borrow::Cow;

use serde::{Deserialize, Deserializer};
use url::Url;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub aws_access_key_id: Cow<'static, str>,
    pub aws_secret_access_key: Cow<'static, str>,
    #[serde(default = "default_aws_region")]
    pub aws_region: Cow<'static, str>,
    #[serde(default = "default_bucket_name")]
    pub bucket_name: Cow<'static, str>,
    #[serde(default = "default_bucket_endpoint_url")]
    pub bucket_endpoint_url: Url,
    #[serde(deserialize_with = "string_to_vec", default = "default_origin_urls")]
    pub origin_urls: Vec<String>,
    #[serde(default = "default_refresh_enabled")]
    pub refresh_enabled: bool,
    #[serde(default = "default_cron_refresh")]
    pub cron_refresh: String,
    #[serde(default = "default_api_prefix")]
    pub api_prefix: String,
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

fn default_port() -> u16 {
    8000
}

fn default_aws_region() -> Cow<'static, str> {
    Cow::Borrowed("us-east-1")
}

fn default_bucket_name() -> Cow<'static, str> {
    Cow::Borrowed("colette")
}

fn default_bucket_endpoint_url() -> Url {
    "http://localhost:9000".parse().unwrap()
}

pub fn string_to_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    let parsed = value.split(',').map(|s| s.trim().to_owned()).collect();

    Ok(parsed)
}

fn default_origin_urls() -> Vec<String> {
    Vec::new()
}

fn default_refresh_enabled() -> bool {
    true
}

fn default_cron_refresh() -> String {
    "0 */15 * * * *".to_owned()
}

fn default_api_prefix() -> String {
    "/api/v1".to_owned()
}

pub fn load_config() -> Result<AppConfig, envy::Error> {
    envy::from_env::<AppConfig>()
}
