// We implement here a gRPC middleware to provide access for the Service
// object inside every RPC method.

pub mod rpc;

use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::{body::BoxBody, transport::Body};
use tower::{Layer, Service};

use crate::service;

#[derive(Debug, Clone)]
pub(crate) struct GrpcMiddleware {
    service: Arc<service::Service>,
}

impl GrpcMiddleware {
    pub(crate) fn new(service: &Arc<service::Service>) -> Self {
        GrpcMiddleware {
            service: service.clone(),
        }
    }
}

impl<S> Layer<S> for GrpcMiddleware {
    type Service = MicroServiceGrpcMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MicroServiceGrpcMiddleware {
            inner: service,
            service: self.service.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MicroServiceGrpcMiddleware<S> {
    inner: S,
    service: Arc<service::Service>,
}

impl<S> Service<http::request::Request<Body>> for MicroServiceGrpcMiddleware<S>
where
    S: Service<http::request::Request<Body>, Response = http::response::Response<BoxBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        req.extensions_mut().insert(self.service.clone());
        Box::pin(async move {
            let response = inner.call(req).await?;
            Ok(response)
        })
    }
}
