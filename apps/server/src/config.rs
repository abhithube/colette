use std::path::PathBuf;

use chrono::Duration;
use url::Url;

const APP_NAME: &str = "colette";

pub fn from_env() -> Result<Config, envy::Error> {
    let raw = envy::from_env::<RawConfig>()?;
    let config = raw.try_into()?;

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub jwt: JwtConfig,
    pub queue: QueueConfig,
    pub storage: StorageConfig,
    pub oidc: Option<OidcConfig>,
    pub cron: Option<CronConfig>,
    pub cors: Option<CorsConfig>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 8000 }
    }
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_duration: Duration,
    pub refresh_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum QueueConfig {
    Local,
}

#[derive(Debug, Clone)]
pub enum StorageConfig {
    Fs(FsStorageConfig),
    S3(S3StorageConfig),
}

#[derive(Debug, Clone)]
pub struct FsStorageConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct S3StorageConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint: Url,
    pub bucket_name: String,
    pub path_style: S3RequestStyle,
}

#[derive(Debug, Clone, Default)]
pub enum S3RequestStyle {
    #[default]
    Path,
    VirtualHost,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub client_id: String,
    pub discovery_endpoint: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct CronConfig {
    pub schedule: String,
}

impl Default for CronConfig {
    fn default() -> Self {
        CronConfig {
            schedule: "0 */15 * * * *".into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CorsConfig {
    pub origin_urls: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawConfig {
    data_dir: Option<PathBuf>,
    server_port: Option<u32>,
    database_url: String,
    jwt_secret: String,
    jwt_issuer: Option<String>,
    #[serde(default = "jwt_access_duration")]
    jwt_access_duration: Duration,
    #[serde(default = "jwt_refresh_duration")]
    jwt_refresh_duration: Duration,
    oidc_client_id: Option<String>,
    oidc_discovery_endpoint: Option<String>,
    oidc_redirect_uri: Option<String>,
    #[serde(default = "QueueBackend::default")]
    queue_backend: QueueBackend,
    #[serde(default = "StorageBackend::default")]
    storage_backend: StorageBackend,
    aws_access_key_id: Option<String>,
    aws_secret_access_key: Option<String>,
    aws_region: Option<String>,
    aws_endpoint: Option<Url>,
    s3_bucket_name: Option<String>,
    #[serde(default = "s3_path_style_enabled")]
    s3_path_style_enabled: bool,
    #[serde(default = "cron_enabled")]
    cron_enabled: bool,
    cron_schedule: Option<String>,
    #[serde(default = "cors_enabled")]
    cors_enabled: bool,
    cors_origin_urls: Option<Vec<String>>,
}

impl TryFrom<RawConfig> for Config {
    type Error = envy::Error;

    fn try_from(value: RawConfig) -> Result<Self, Self::Error> {
        let data_dir = value
            .data_dir
            .unwrap_or_else(|| dirs::config_dir().unwrap().join(APP_NAME));

        std::fs::create_dir_all(&data_dir).unwrap();

        let mut server = ServerConfig::default();
        if let Some(port) = value.server_port {
            server.port = port;
        }

        let jwt = JwtConfig {
            secret: value.jwt_secret,
            issuer: value
                .jwt_issuer
                .unwrap_or_else(|| format!("http://0.0.0.0:{}", server.port)),
            audience: APP_NAME.into(),
            access_duration: value.jwt_access_duration,
            refresh_duration: value.jwt_refresh_duration,
        };

        let queue = match value.queue_backend {
            QueueBackend::Local => QueueConfig::Local,
        };

        let storage = match value.storage_backend {
            StorageBackend::Fs => {
                let config = FsStorageConfig {
                    path: data_dir.join("storage"),
                };
                if !std::fs::exists(&config.path).unwrap() {
                    let _ = std::fs::create_dir(&config.path);
                }

                StorageConfig::Fs(config)
            }
            StorageBackend::S3 => {
                let access_key_id = value
                    .aws_access_key_id
                    .expect("'AWS_ACCESS_KEY_ID' not specified");
                let secret_access_key = value
                    .aws_secret_access_key
                    .expect("'AWS_SECRET_ACCESS_KEY' not specified");
                let endpoint = value.aws_endpoint.expect("'AWS_ENDPOINT' not specified");

                let config = S3StorageConfig {
                    access_key_id,
                    secret_access_key,
                    region: value.aws_region.unwrap_or_else(|| "us-east-1".into()),
                    endpoint,
                    bucket_name: value.s3_bucket_name.unwrap_or_else(|| APP_NAME.into()),
                    path_style: if value.s3_path_style_enabled {
                        S3RequestStyle::Path
                    } else {
                        S3RequestStyle::VirtualHost
                    },
                };

                StorageConfig::S3(config)
            }
        };

        let mut cron = None;
        if value.cron_enabled {
            let mut config = CronConfig::default();
            if let Some(schedule) = value.cron_schedule {
                config.schedule = schedule;
            }

            cron = Some(config);
        }

        let mut cors = None;
        if value.cors_enabled {
            let mut config = CorsConfig::default();
            if let Some(origin_urls) = value.cors_origin_urls {
                config.origin_urls = origin_urls;
            }

            cors = Some(config);
        }

        let mut oidc = None::<OidcConfig>;
        if let (Some(client_id), Some(redirect_uri), Some(discovery_endpoint)) = (
            value.oidc_client_id,
            value.oidc_redirect_uri,
            value.oidc_discovery_endpoint,
        ) {
            oidc = Some(OidcConfig {
                client_id,
                redirect_uri,
                discovery_endpoint,
            })
        }

        Ok(Self {
            server,
            database_url: value.database_url,
            jwt,
            queue,
            storage,
            oidc,
            cron,
            cors,
        })
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueueBackend {
    #[default]
    Local,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageBackend {
    #[default]
    Fs,
    S3,
}

fn jwt_access_duration() -> Duration {
    Duration::minutes(15)
}

fn jwt_refresh_duration() -> Duration {
    Duration::days(7)
}

fn s3_path_style_enabled() -> bool {
    true
}

fn cron_enabled() -> bool {
    true
}

fn cors_enabled() -> bool {
    false
}
