use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub redis_url: Option<String>,
    #[serde(deserialize_with = "string_to_vec", default = "default_origin_urls")]
    pub origin_urls: Vec<String>,
    #[serde(default = "default_refresh_enabled")]
    pub refresh_enabled: bool,
    #[serde(default = "default_cron_refresh")]
    pub cron_refresh: String,
}

fn default_host() -> String {
    String::from("0.0.0.0")
}

fn default_port() -> u16 {
    8000
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
    vec![]
}

fn default_refresh_enabled() -> bool {
    true
}

fn default_cron_refresh() -> String {
    String::from("0 */15 * * * *")
}

pub fn load_config() -> Result<Config, envy::Error> {
    envy::from_env::<Config>()
}
