use serde_derive::Deserialize;
use validator::Validate;

mod validation;

use crate::definition::validation::service_kind_oneof;
use crate::error::{Error, Result};

#[derive(Debug)]
pub(crate) struct ServiceDefinition {
    pub info: ServiceInfo,
}

#[derive(Debug, Deserialize, Validate)]
pub(crate) struct ServiceInfo {
    pub name: String,
    pub version: String,

    #[validate(custom(function = "service_kind_oneof"))]
    #[serde(rename(deserialize = "type"))]
    pub kind: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) enum ServiceKind {
    Unsupported,
    Grpc,
    Http,
    Pubsub,
}

impl ServiceDefinition {
    pub fn new() -> Result<Self> {
        let info: ServiceInfo = match toml::from_str(&Self::load_settings_file()?) {
            Ok(content) => content,
            Err(e) => return Err(Error::DefinitionParser(e.to_string())),
        };

        if let Err(e) = info.validate() {
            return Err(Error::UnsupportedSetting(e.to_string()));
        }

        Ok(ServiceDefinition { info })
    }

    fn load_settings_file() -> Result<String> {
        let path = Self::get_settings_file_path()?;

        match std::fs::read_to_string(path.as_path()) {
            Ok(content) => Ok(content),
            Err(e) => Err(Error::InternalOS(e.to_string())),
        }
    }

    fn get_settings_file_path() -> Result<std::path::PathBuf> {
        match std::env::current_dir() {
            Ok(mut p) => {
                p.push("service.toml");
                Ok(p)
            }
            Err(r) => Err(Error::InternalOS(r.to_string())),
        }
    }
}

impl ServiceKind {
    pub fn from_str(value: &str) -> ServiceKind {
        match value {
            "grpc" => ServiceKind::Grpc,
            "http" => ServiceKind::Http,
            "pubsub" => ServiceKind::Pubsub,
            _ => ServiceKind::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_service_info_new() {}
}
