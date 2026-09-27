use std::fmt;

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ErrorCode {
    MissingCredentials,
    InvalidCredentials,
    NotDesktopClient,
    LoginTimeout,
    LoginMismatch,
    LoginDenied,
    NoRefreshToken,
    TokenRejected,
    NoSession,
    Network,
    Gmail,
    GmailBusy,
    ClaudeMissing,
    ClaudeFailed,
    InvalidLabel,
    InvalidSender,
    InvalidId,
    InvalidLocale,
    Keyring,
    Io,
    InvalidData,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Error {
    pub code: ErrorCode,
    pub detail: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: ErrorCode) -> Self {
        Self { code, detail: None }
    }

    pub fn with(code: ErrorCode, detail: impl fmt::Display) -> Self {
        Self {
            code,
            detail: Some(detail.to_string()),
        }
    }
}

impl From<ErrorCode> for Error {
    fn from(code: ErrorCode) -> Self {
        Self::new(code)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail.as_deref().unwrap_or_default())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::with(ErrorCode::Io, e)
    }
}

impl From<ureq::Error> for Error {
    fn from(e: ureq::Error) -> Self {
        Self::with(ErrorCode::Network, e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::with(ErrorCode::InvalidData, e)
    }
}

impl From<keyring::Error> for Error {
    fn from(e: keyring::Error) -> Self {
        Self::with(ErrorCode::Keyring, e)
    }
}
