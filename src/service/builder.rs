use crate::error::Result;
use crate::service::Service;
use std::sync::Arc;

const SERVICE_PORT: i64 = 9090;

pub struct ServiceBuilder {
    pub(crate) port: i64,
}

impl ServiceBuilder {
    fn new() -> Self {
        ServiceBuilder { port: SERVICE_PORT }
    }

    pub fn with_port(&mut self, port: i64) -> &mut Self {
        self.port = port;
        self
    }

    pub fn build(&mut self) -> Result<Arc<Service>> {
        Service::new(self)
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
