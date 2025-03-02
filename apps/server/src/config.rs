use std::path::PathBuf;

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
    pub database: DatabaseConfig,
    pub job: JobConfig,
    pub session: SessionConfig,
    pub storage: StorageConfig,
    pub cron: Option<CronConfig>,
    pub cors: Option<CorsConfig>,
}

#[derive(Debug, Clone)]
pub struct SqliteConfig {
    pub url: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FsConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint: Url,
    pub bucket_name: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            access_key_id: "minioadmin".into(),
            secret_access_key: "minioadmin".into(),
            region: "us-east-1".into(),
            endpoint: "http://localhost:9000".parse().unwrap(),
            bucket_name: APP_NAME.into(),
        }
    }
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
pub enum DatabaseConfig {
    Sqlite(SqliteConfig),
}

#[derive(Debug, Clone)]
pub enum SessionConfig {
    Sqlite(SqliteConfig),
    Redis(RedisConfig),
}

#[derive(Debug, Clone)]
pub enum JobConfig {
    Sqlite(SqliteConfig),
}

#[derive(Debug, Clone)]
pub enum StorageConfig {
    Fs(FsConfig),
    S3(S3Config),
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

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct RawConfig {
    data_dir: Option<PathBuf>,
    server_port: Option<u32>,
    #[serde(default = "DatabaseBackend::default")]
    database_backend: DatabaseBackend,
    #[serde(default = "SessionBackend::default")]
    session_backend: SessionBackend,
    #[serde(default = "JobBackend::default")]
    job_backend: JobBackend,
    #[serde(default = "StorageBackend::default")]
    storage_backend: StorageBackend,
    redis_url: Option<String>,
    aws_access_key_id: Option<String>,
    aws_secret_access_key: Option<String>,
    aws_region: Option<String>,
    s3_bucket_name: Option<String>,
    s3_endpoint: Option<Url>,
    #[serde(default = "cron_enabled")]
    cron_enabled: bool,
    cron_schedule: Option<String>,
    #[serde(default = "cors_enabled")]
    cors_enabled: bool,
    cors_origin_urls: Vec<String>,
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

        let database = match value.database_backend {
            DatabaseBackend::Sqlite => {
                let config = SqliteConfig {
                    url: data_dir.join("db.sqlite"),
                };

                DatabaseConfig::Sqlite(config)
            }
        };

        let session = match value.session_backend {
            SessionBackend::Sqlite => {
                let config = SqliteConfig {
                    url: data_dir.join("session.sqlite"),
                };

                SessionConfig::Sqlite(config)
            }
            SessionBackend::Redis => {
                let mut config = RedisConfig::default();
                if let Some(url) = value.redis_url {
                    config.url = url;
                }

                SessionConfig::Redis(config)
            }
        };

        let job = match value.job_backend {
            JobBackend::Sqlite => {
                let config = SqliteConfig {
                    url: data_dir.join("job.sqlite"),
                };

                JobConfig::Sqlite(config)
            }
        };

        let storage = match value.storage_backend {
            StorageBackend::Fs => {
                let config = FsConfig {
                    path: data_dir.join("storage"),
                };
                if !std::fs::exists(&config.path).unwrap() {
                    let _ = std::fs::create_dir(&config.path);
                }

                StorageConfig::Fs(config)
            }
            StorageBackend::S3 => {
                let mut config = S3Config::default();
                if let Some(access_key_id) = value.aws_access_key_id {
                    config.access_key_id = access_key_id;
                }
                if let Some(secret_access_key) = value.aws_secret_access_key {
                    config.secret_access_key = secret_access_key;
                }
                if let Some(region) = value.aws_region {
                    config.region = region;
                }
                if let Some(endpoint) = value.s3_endpoint {
                    config.endpoint = endpoint;
                }
                if let Some(bucket_name) = value.s3_bucket_name {
                    config.bucket_name = bucket_name;
                }

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
            if !value.cors_origin_urls.is_empty() {
                config.origin_urls = value.cors_origin_urls;
            }

            cors = Some(config);
        }

        let config = Self {
            server,
            database,
            session,
            job,
            storage,
            cron,
            cors,
        };

        Ok(config)
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    #[default]
    Sqlite,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionBackend {
    #[default]
    Sqlite,
    Redis,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobBackend {
    #[default]
    Sqlite,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    #[default]
    Fs,
    S3,
}

fn cron_enabled() -> bool {
    true
}

fn cors_enabled() -> bool {
    false
}
