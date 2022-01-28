use crate::database::{Credentials, Info};
use crate::error::Result;
use crate::service::Service;
use std::sync::Arc;

const SERVICE_PORT: i64 = 9090;

pub struct ServiceBuilder {
    pub(crate) port: i64,
    pub(crate) credentials: Credentials,
    pub(crate) db_info: Info,
}

impl ServiceBuilder {
    fn new() -> Self {
        ServiceBuilder {
            port: SERVICE_PORT,
            credentials: Credentials::default(),
            db_info: Info::default(),
        }
    }

    pub fn with_port(&mut self, port: i64) -> &mut Self {
        self.port = port;
        self
    }

    pub fn with_database_info(&mut self, info: &Info) -> &mut Self {
        self.db_info = info.clone();
        self
    }

    pub fn with_database_credentials(&mut self, credentials: &Credentials) -> &mut Self {
        self.credentials = credentials.clone();
        self
    }

    pub async fn build(&mut self) -> Result<Arc<Service>> {
        Service::new(self).await
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
