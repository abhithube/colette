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
    pub queue: QueueConfig,
    pub storage: StorageConfig,
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
pub enum DatabaseConfig {
    Libsql(LibsqlConfig),
}

#[derive(Debug, Clone)]
pub struct LibsqlConfig {
    pub url: PathBuf,
}

#[derive(Debug, Clone)]
pub enum QueueConfig {
    Local,
}

#[derive(Debug, Clone)]
pub enum StorageConfig {
    Local(LocalStorageConfig),
    // S3(S3StorageConfig),
}

#[derive(Debug, Clone)]
pub struct LocalStorageConfig {
    pub path: PathBuf,
}

// #[derive(Debug, Clone)]
// pub struct S3StorageConfig {
//     pub access_key_id: String,
//     pub secret_access_key: String,
//     pub region: String,
//     pub endpoint: Url,
//     pub bucket_name: String,
// }

// impl Default for S3StorageConfig {
//     fn default() -> Self {
//         Self {
//             access_key_id: "minioadmin".into(),
//             secret_access_key: "minioadmin".into(),
//             region: "us-east-1".into(),
//             endpoint: "http://localhost:9000".parse().unwrap(),
//             bucket_name: APP_NAME.into(),
//         }
//     }
// }

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
    #[serde(default = "QueueBackend::default")]
    queue_backend: QueueBackend,
    #[serde(default = "StorageBackend::default")]
    storage_backend: StorageBackend,
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

        let database = match value.database_backend {
            DatabaseBackend::Libsql => {
                let config = LibsqlConfig {
                    url: data_dir.join("db.sqlite"),
                };

                DatabaseConfig::Libsql(config)
            }
        };

        let queue = match value.queue_backend {
            QueueBackend::Local => QueueConfig::Local,
        };

        let storage = match value.storage_backend {
            StorageBackend::Local => {
                let config = LocalStorageConfig {
                    path: data_dir.join("storage"),
                };
                if !std::fs::exists(&config.path).unwrap() {
                    let _ = std::fs::create_dir(&config.path);
                }

                StorageConfig::Local(config)
            } // StorageBackend::S3 => {
              //     let mut config = S3StorageConfig::default();
              //     if let Some(access_key_id) = value.aws_access_key_id {
              //         config.access_key_id = access_key_id;
              //     }
              //     if let Some(secret_access_key) = value.aws_secret_access_key {
              //         config.secret_access_key = secret_access_key;
              //     }
              //     if let Some(region) = value.aws_region {
              //         config.region = region;
              //     }
              //     if let Some(endpoint) = value.s3_endpoint {
              //         config.endpoint = endpoint;
              //     }
              //     if let Some(bucket_name) = value.s3_bucket_name {
              //         config.bucket_name = bucket_name;
              //     }

              //     StorageConfig::S3(config)
              // }
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

        Ok(Self {
            server,
            database,
            queue,
            storage,
            cron,
            cors,
        })
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatabaseBackend {
    #[default]
    Libsql,
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
    Local,
    // S3,
}

fn cron_enabled() -> bool {
    true
}

fn cors_enabled() -> bool {
    false
}
