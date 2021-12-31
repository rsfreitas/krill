pub mod builder;

use crate::config::{Config, ConfigBuilder, GetEnv};
use crate::definition::{ServiceDefinition, ServiceKind};
use crate::error::Result;
use logger::{builder::LoggerBuilder, fields::FieldValue, Logger};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures_util::FutureExt;
use http::{request::Request, response::Response};
use tokio::signal;
use tonic::body::BoxBody;
use tonic::transport::{Body, NamedService};

use crate::grpc;
use builder::ServiceBuilder;

#[derive(Debug)]
pub struct Service {
    pub logger: Arc<Logger>,
    pub config: Config,

    name: String,

    #[allow(dead_code)]
    kind: ServiceKind,
    port: i64,
}

impl Service {
    fn new(builder: &ServiceBuilder) -> Result<Arc<Self>> {
        let definition = ServiceDefinition::new()?;
        let logger = Arc::new(
            LoggerBuilder::new()
                .with_field(
                    "service.name",
                    FieldValue::String(definition.info.name.clone()),
                )
                .with_field(
                    "service.version",
                    FieldValue::String(definition.info.version.clone()),
                )
                .with_field(
                    "service.type",
                    FieldValue::String(definition.info.kind.to_string()),
                )
                .build(),
        );

        logger.info("starting service");

        Ok(Arc::new(Service {
            name: definition.info.name.clone(),
            kind: ServiceKind::from_str(&definition.info.kind),
            config: ConfigBuilder::new().with_logger(&logger).build(),
            logger: logger.clone(),
            port: Service::get_service_port(builder),
        }))
    }

    fn get_service_port(builder: &ServiceBuilder) -> i64 {
        match Config::get_os_env("SERVICE_PORT") {
            None => builder.port,
            Some(port) => port,
        }
    }

    /// Gives back the current service name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieves the Service object from RPC's request argument.
    pub fn from_request<B: prost::Message>(request: tonic::Request<B>) -> Arc<Service> {
        request.extensions().get::<Arc<Service>>().unwrap().clone()
    }

    /// Puts the service to run in the gRPC mode.
    pub async fn serve_as_grpc<S>(
        service: &Arc<Service>,
        grpc_server: S,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>
    where
        S: tower_service::Service<Request<Body>, Response = Response<BoxBody>>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    {
        service.service_internals_start();

        let layer = tower::ServiceBuilder::new()
            .timeout(Duration::from_secs(30))
            .layer(grpc::GrpcMiddleware::new(service))
            .into_inner();

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            service.port.try_into().unwrap(),
        );

        service.logger.infof(
            "service is running",
            logger::fields! {
                "service.address" => FieldValue::String(format!(":{}", service.port)),
            },
        );

        let jh = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .layer(layer)
                .add_service(grpc_server)
                .serve_with_shutdown(addr, shutdown_rx.map(drop))
                .await
                .unwrap();
        });

        tokio::spawn(async move {
            let _ = signal::ctrl_c().await;
            shutdown_tx
                .send(())
                .expect("could not send signal to finish service");
        });

        jh.await.expect("could not receive shutdown signal");
        Ok(())
    }

    fn service_internals_start(&self) {
        // TODO: database init
    }

    /// Stops the service. This method is called when the Service object is
    /// dropped.
    pub fn stop(&self) {
        self.logger.info("stopping service");
    }

    /// Returns error from a gRPC method.
    pub fn error<R: prost::Message>(
        error: grpc::Error,
    ) -> std::result::Result<tonic::Response<R>, tonic::Status> {
        Err(error.to_status(""))
    }

    /// Returns error from a gRPC method with a custom message.
    pub fn error_with_message<R: prost::Message>(
        error: grpc::Error,
        msg: &str,
    ) -> std::result::Result<tonic::Response<R>, tonic::Status> {
        Err(error.to_status(msg))
    }

    /// Returns success from a gRPC method.
    pub fn ok<R: prost::Message>(res: R) -> std::result::Result<tonic::Response<R>, tonic::Status> {
        Ok(tonic::Response::new(res))
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.stop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_service_new() {
        let svc = ServiceBuilder::default().build();
        svc.unwrap().start();
    }
}
