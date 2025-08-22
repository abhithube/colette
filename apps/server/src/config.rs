use config::{Config, Environment, FileFormat};
use url::Url;

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

#[cfg(debug_assertions)]
const DEVELOPMENT_CONFIG: &str = include_str!("../config/development.toml");

pub async fn from_env() -> Result<AppConfig, Box<dyn std::error::Error>> {
    #[allow(unused_mut)]
    let mut builder = Config::builder()
        .add_source(config::File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
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

    let database = DatabaseConfig {
        url: raw.database.url,
    };

    let jwt = JwtConfig {
        secret: raw.jwt.secret,
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
            issuer_url: oidc.issuer_url.expect("'OIDC__ISSUER_URL' not set"),
            client_id: oidc.client_id.expect("'OIDC__CLIENT_ID' not set"),
            redirect_uri: redirect_uri.into(),
            scopes: oidc.scopes,
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
        s3,
        oidc,
    })
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub cors: Option<CorsConfig>,
    pub s3: S3Config,
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
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub origin_urls: Vec<Url>,
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
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub sign_in_text: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawConfig {
    server: ServerConfig,
    database: RawDatabaseConfig,
    client: Option<ClientConfig>,
    jwt: RawJwtConfig,
    cors: RawCorsConfig,
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
struct RawS3Config {
    access_key_id: String,
    secret_access_key: String,
    region: String,
    endpoint: String,
    bucket_name: String,
    path_style_enabled: bool,
    image_base_url: Option<Url>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawOidcConfig {
    enabled: bool,
    client_id: Option<String>,
    issuer_url: Option<String>,
    scopes: Vec<String>,
    sign_in_text: String,
}
