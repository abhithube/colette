use std::{path::PathBuf, sync::LazyLock};

use config::{Config, Environment, FileFormat};
use tokio::fs::{self, File};
use url::Url;

const APP_NAME: &str = "Colette";
static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::config_dir().unwrap().join(APP_NAME.to_lowercase()));

const SQLITE_PATH: &str = "sqlite/db.sqlite";
const FS_PATH: &str = "fs";

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

#[cfg(debug_assertions)]
const DEVELOPMENT_CONFIG: &str = include_str!("../config/development.toml");

pub async fn from_env() -> Result<AppConfig, Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut builder = Config::builder()
        .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        .set_default("data_dir", DATA_DIR.to_string_lossy().into_owned())?
        .add_source(
            Environment::default()
                .separator("__")
                .list_separator(",")
                .with_list_parse_key("cors.origin_urls")
                .try_parsing(true),
        );

    #[cfg(debug_assertions)]
    {
        builder = builder.add_source(config::File::from_str(DEVELOPMENT_CONFIG, FileFormat::Toml));
    }

    let raw = builder.build()?.try_deserialize::<RawConfig>()?;

    let database = match raw.database.map(|e| e.url) {
        Some(url) => DatabaseConfig::Postgres(PostgresConfig { url }),
        None => {
            let path = raw.data_dir.join(SQLITE_PATH);

            if !fs::try_exists(&path).await? {
                if let Some(prefix) = path.parent() {
                    fs::create_dir_all(prefix).await?;
                }

                File::create(&path).await?;
            }

            DatabaseConfig::Sqlite(SqliteConfig { path })
        }
    };

    let jwt = JwtConfig {
        secret: raw.jwt.secret,
        issuer: raw.server.base_url.to_string(),
        audience: vec![APP_NAME.to_lowercase()],
    };

    let cors = raw.cors.enabled.then(|| {
        let mut origin_urls = raw.cors.origin_urls;
        if let Some(ref config) = raw.client
            && !origin_urls.contains(&config.base_url)
        {
            origin_urls.push(config.base_url.to_owned());
        }

        CorsConfig { origin_urls }
    });

    let storage = match raw.storage.backend {
        RawStorageBackend::Fs => {
            let path = raw.data_dir.join(FS_PATH);

            if !tokio::fs::try_exists(&path).await? {
                fs::create_dir_all(&path).await?;
            }

            let mut image_base_url = raw.server.base_url.clone();
            image_base_url.set_path("uploads");

            StorageConfig {
                image_base_url,
                backend: StorageBackend::Fs(FsConfig { path }),
            }
        }
        RawStorageBackend::S3 => {
            let access_key_id = raw.s3.access_key_id.expect("'AWS__ACCESS_KEY_ID' not set");
            let secret_access_key = raw
                .s3
                .secret_access_key
                .expect("'AWS__SECRET_ACCESS_KEY' not set");
            let region = raw.s3.region.expect("'AWS__REGION' not set");
            let endpoint = raw.s3.endpoint.expect("'AWS__ENDPOINT' not set");

            let image_base_url = raw.s3.image_base_url.unwrap_or_else(|| {
                let mut image_base_url = endpoint.clone();

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

            StorageConfig {
                image_base_url,
                backend: StorageBackend::S3(S3Config {
                    access_key_id,
                    secret_access_key,
                    region,
                    endpoint,
                    bucket_name: raw.s3.bucket_name,
                    path_style_enabled: raw.s3.path_style_enabled,
                }),
            }
        }
    };

    let oidc = if let Some(oidc) = raw.oidc
        && oidc.enabled
    {
        let mut redirect_uri = if let Some(ref config) = raw.client {
            config.base_url.clone()
        } else {
            raw.server.base_url.clone()
        };

        redirect_uri.set_path("auth-callback");

        Some(OidcConfig {
            client_id: oidc.client_id.expect("'OIDC__CLIENT_ID' not set"),
            discovery_endpoint: oidc
                .discovery_endpoint
                .expect("'OIDC__DISCOVERY_ENDPOINT' not set"),
            redirect_uri: redirect_uri.into(),
            scope: oidc.scope,
            sign_in_text: oidc.sign_in_text,
        })
    } else {
        None
    };

    Ok(AppConfig {
        server: raw.server,
        database,
        jwt,
        cors,
        storage,
        oidc,
    })
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub cors: Option<CorsConfig>,
    pub storage: StorageConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub port: u32,
    pub base_url: Url,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {
    pub base_url: Url,
}

#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Sqlite(SqliteConfig),
    Postgres(PostgresConfig),
}

#[derive(Debug, Clone)]
pub struct SqliteConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub origin_urls: Vec<Url>,
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub image_base_url: Url,
    pub backend: StorageBackend,
}

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Fs(FsConfig),
    S3(S3Config),
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
    pub path_style_enabled: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OidcConfig {
    pub client_id: String,
    pub discovery_endpoint: Url,
    pub redirect_uri: String,
    pub scope: String,
    pub sign_in_text: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawConfig {
    data_dir: PathBuf,
    server: ServerConfig,
    database: Option<RawDatabaseConfig>,
    client: Option<ClientConfig>,
    jwt: RawJwtConfig,
    cors: RawCorsConfig,
    storage: RawStorageConfig,
    s3: RawS3Config,
    oidc: Option<RawOidcConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawDatabaseConfig {
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawJwtConfig {
    secret: String,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct RawCorsConfig {
    enabled: bool,
    origin_urls: Vec<Url>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawStorageConfig {
    backend: RawStorageBackend,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawStorageBackend {
    Fs,
    S3,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawS3Config {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    region: Option<String>,
    endpoint: Option<Url>,
    bucket_name: String,
    path_style_enabled: bool,
    image_base_url: Option<Url>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawOidcConfig {
    enabled: bool,
    client_id: Option<String>,
    discovery_endpoint: Option<Url>,
    scope: String,
    sign_in_text: String,
}
