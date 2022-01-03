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

/// Returns error from a gRPC method.
pub fn error<R: prost::Message>(
    error: ErrorCode,
) -> std::result::Result<tonic::Response<R>, tonic::Status> {
    Err(Error::new(error, None).to_status())
}

/// Returns error from a gRPC method with a custom message.
pub fn error_with_message<R: prost::Message>(
    error: ErrorCode,
    msg: &str,
) -> std::result::Result<tonic::Response<R>, tonic::Status> {
    Err(Error::new(error, Some(msg)).to_status())
}

/// Returns success from a gRPC method.
pub fn ok<R: prost::Message>(res: R) -> std::result::Result<tonic::Response<R>, tonic::Status> {
    Ok(tonic::Response::new(res))
}

