use logger::{fields::FieldValue, Logger};
use std::sync::Arc;

#[derive(Debug)]
pub struct Config {
    logger: Option<Arc<Logger>>,
}

#[derive(Debug)]
pub(crate) struct ConfigBuilder {
    logger: Option<Arc<Logger>>,
}

pub(crate) trait GetEnv<T> {
    fn get_os_env(key: &str, default: Option<T>) -> Option<T>;
}

impl Config {
    fn new(builder: &ConfigBuilder) -> Self {
        Config {
            logger: builder.logger.as_ref().cloned(),
        }
    }

    pub fn get_env(&self, key: &str, default_value: &str) -> Option<String> {
        match std::env::var(key.to_string()) {
            Err(_) => {
                if !default_value.is_empty() {
                    Some(default_value.to_string())
                } else {
                    None
                }
            }
            Ok(value) => Some(value),
        }
    }

    pub fn must_get_env(&self, key: &str) -> Option<String> {
        let k = key.to_string();
        match std::env::var(&k) {
            Err(_) => {
                if let Some(log) = &self.logger {
                    log.debugf(
                        "environment variable must be set",
                        logger::fields!("name" => FieldValue::String(k)),
                    );
                }

                None
            }
            Ok(v) => Some(v),
        }
    }
}

impl GetEnv<String> for Config {
    fn get_os_env(key: &str, default: Option<String>) -> Option<String> {
        match std::env::var(key.to_string()) {
            Err(_) => default,
            Ok(value) => Some(value),
        }
    }
}

impl GetEnv<i64> for Config {
    fn get_os_env(key: &str, default: Option<i64>) -> Option<i64> {
        match std::env::var(key.to_string()) {
            Err(_) => default,
            Ok(value) => match value.parse::<i64>() {
                Err(_) => default,
                Ok(i) => Some(i),
            },
        }
    }
}

impl GetEnv<i32> for Config {
    fn get_os_env(key: &str, default: Option<i32>) -> Option<i32> {
        match std::env::var(key.to_string()) {
            Err(_) => default,
            Ok(value) => match value.parse::<i32>() {
                Err(_) => default,
                Ok(i) => Some(i),
            },
        }
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder { logger: None }
    }

    pub fn with_logger(&mut self, logger: &Arc<Logger>) -> &mut Self {
        self.logger = Some(logger.clone());
        self
    }

    pub fn build(&self) -> Config {
        Config::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logger::builder::LoggerBuilder;

    #[test]
    pub fn test_config_new() {
        let log = Arc::new(LoggerBuilder::default().build());
        let config = ConfigBuilder::new().with_logger(&log).build();
        config.must_get_env("TEST");
        let config2 = ConfigBuilder::new().with_logger(&log).build();
        config2.must_get_env("TEST2");
        let config3 = ConfigBuilder::new().build();
        config3.must_get_env("TEST3");
        log.info("Test");
    }
}
