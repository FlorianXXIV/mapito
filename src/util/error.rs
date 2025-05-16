use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum ApiErrorKind {
    NotFound,
    InvalidVersion,
    InvalidLoader,
    InvalidData,
}

#[derive(Debug)]
pub struct ApiError {
    kind: ApiErrorKind,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_display = match self.kind {
            ApiErrorKind::NotFound => "NotFound",
            ApiErrorKind::InvalidVersion => "InvalidVersion",
            ApiErrorKind::InvalidLoader => "InvalidLoader",
            ApiErrorKind::InvalidData => "InvalidData",
        };
        write!(f, "{}", to_display)
    }
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    //fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
}

impl ApiError {
    pub fn not_found() -> Self {
        ApiError { kind: ApiErrorKind::NotFound }
    }

    pub fn invalid_loader() -> Self {
        ApiError { kind: ApiErrorKind::InvalidLoader }
    }

    pub fn invalid_version() -> Self {
        ApiError { kind: ApiErrorKind::InvalidVersion }
    }

    pub fn invalid_data() -> Self {
        ApiError { kind: ApiErrorKind::InvalidData }
    }
}
