use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum ApiErrorKind {
    NotFound,
    ReqwestError(reqwest::Error)
}

#[derive(Debug)]
pub struct ApiError {
    kind: ApiErrorKind,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_display = match &self.kind {
            ApiErrorKind::NotFound => "NotFound",
            ApiErrorKind::ReqwestError(e) => &e.to_string()
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

}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        ApiError { kind: ApiErrorKind::ReqwestError(value) }
    }
}
