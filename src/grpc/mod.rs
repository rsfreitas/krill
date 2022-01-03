// We implement here a gRPC middleware to provide access for the Service
// object inside every RPC method.

use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::{body::BoxBody, transport::Body};
use tower::{Layer, Service};

use crate::service;

/// Response is an alias for RPC's methods result type.
pub type Response<B> = std::result::Result<tonic::Response<B>, tonic::Status>;

/// Request is an alias for RPC's methods request argument type.
pub type Request<B> = tonic::Request<B>;

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: Option<String>,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = format!("code:{}", self.code);

        if let Some(msg) = &self.message {
            s = format!("{} message:{}", s, msg);
        }

        write!(f, "{}", s)
    }
}

impl Error {
    pub(crate) fn new(code: ErrorCode, msg: Option<&str>) -> Self {
        Error {
            code,
            message: msg.map(|s| s.to_string()),
        }
    }

    pub(crate) fn to_status(&self) -> tonic::Status {
        let code = match self.code {
            ErrorCode::Validation => tonic::Code::InvalidArgument,
            ErrorCode::Internal => tonic::Code::Internal,
            ErrorCode::NotFound => tonic::Code::NotFound,
            ErrorCode::Precondition => tonic::Code::FailedPrecondition,
        };

        tonic::Status::new(code, self.message.clone().unwrap_or_else(|| "".to_string()))
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    Validation,
    Internal,
    NotFound,
    Precondition,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ErrorCode::Validation => write!(f, "InvalidArgument"),
            ErrorCode::Internal => write!(f, "InternalError"),
            ErrorCode::NotFound => write!(f, "NotFound"),
            ErrorCode::Precondition => write!(f, "FailedPrecondition"),
        }
    }
}

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
