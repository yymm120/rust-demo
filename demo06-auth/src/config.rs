use super::error::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct Config {
    // -- Crypt
    pub PWD_KEY: Vec<u8>,

    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,

    // -- Db
    pub DB_URL: String,

    // -- Web
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            // -- Crypt
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,

            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,

            // -- Db
            DB_URL: get_env("SERVICE_DB_URL")?,

            // -- Web
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use serial_test::serial;

    use super::{get_env, get_env_b64u_as_u8s, get_env_parse, Config};

    #[serial]
    #[tokio::test]
    async fn test_config_get_env() -> Result<()> {
        let log = get_env("RUST_LOG")
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"));
        assert_eq!(log, "rust_web_app=debug");
        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_config_get_env_parse() -> Result<()> {
        let duration: i32 = get_env_parse("SERVICE_TOKEN_DURATION_SEC")
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"));
        assert_eq!(duration, 1800);
        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_config_get_env_b64u_as_u8s() -> Result<()> {
        let vec_u8 = get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"));
        let str = base64_url::encode(&vec_u8);
        assert_eq!(str, "9FoHBmkyxbgu_xFoQK7e0jz3RMNVJWgfvbVn712FBNH9LLaAWS3CS6Zpcg6RveiObvCUb6a2z-uAiLjhLh2igw");
        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_config_load_from_env() -> Result<()> {
        let config = Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"));
        assert_eq!(config.WEB_FOLDER, "web-folder/");
        Ok(())
    }
}
