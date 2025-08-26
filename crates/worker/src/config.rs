use config::{Config, Environment, FileFormat};
use url::Url;

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

#[cfg(debug_assertions)]
const DEVELOPMENT_CONFIG: &str = include_str!("../config/development.toml");

pub async fn from_env() -> Result<AppConfig, Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut builder = Config::builder()
        .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        .add_source(Environment::default().separator("__").try_parsing(true));

    #[cfg(debug_assertions)]
    {
        builder = builder.add_source(config::File::from_str(DEVELOPMENT_CONFIG, FileFormat::Toml));
    }

    let raw = builder.build()?.try_deserialize::<RawConfig>()?;

    let database = DatabaseConfig {
        url: raw.database.url,
    };

    let s3 = {
        let image_base_url = raw.s3.image_base_url.unwrap_or_else(|| {
            let mut image_base_url = raw.s3.endpoint.parse::<Url>().unwrap();

            if raw.s3.path_style_enabled {
                image_base_url.set_path(&format!("{}/", raw.s3.bucket_name));
            } else {
                image_base_url
                    .set_host(Some(&format!(
                        "{}.{}",
                        raw.s3.bucket_name,
                        image_base_url.host_str().unwrap()
                    )))
                    .unwrap();
            }

            image_base_url
        });

        S3Config {
            access_key_id: raw.s3.access_key_id,
            secret_access_key: raw.s3.secret_access_key,
            region: raw.s3.region,
            endpoint: raw.s3.endpoint,
            bucket_name: raw.s3.bucket_name,
            path_style_enabled: raw.s3.path_style_enabled,
            image_base_url,
        }
    };

    Ok(AppConfig {
        database,
        smtp: raw.smtp,
        s3,
    })
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub smtp: SmtpConfig,
    pub s3: S3Config,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket_name: String,
    pub path_style_enabled: bool,
    pub image_base_url: Url,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawConfig {
    database: RawDatabaseConfig,
    smtp: SmtpConfig,
    s3: RawS3Config,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawDatabaseConfig {
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawS3Config {
    access_key_id: String,
    secret_access_key: String,
    region: String,
    endpoint: String,
    bucket_name: String,
    path_style_enabled: bool,
    image_base_url: Option<Url>,
}
