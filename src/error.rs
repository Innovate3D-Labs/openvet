use std::fmt;

/// Machine-readable reason a message or a transition was rejected.
///
/// Inbound `error` messages may carry codes that are not in this enum.
/// A peer has to be able to read a rejection it does not have a variant for.
/// This list is only what the reference crate itself produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Malformed,
    UnknownType,
    UnsupportedVersion,
    Limit,
    ProfileUnsupported,
    Conflict,
    Expired,
    WrongState,
    WrongParty,
    BadRevision,
    Overlap,
    Inconsistent,
    NotFiscal,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::UnknownType => "unknown_type",
            Self::UnsupportedVersion => "unsupported_version",
            Self::Limit => "limit",
            Self::ProfileUnsupported => "profile_unsupported",
            Self::Conflict => "conflict",
            Self::Expired => "expired",
            Self::WrongState => "wrong_state",
            Self::WrongParty => "wrong_party",
            Self::BadRevision => "bad_revision",
            Self::Overlap => "overlap",
            Self::Inconsistent => "inconsistent",
            Self::NotFiscal => "not_fiscal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule failed. `code` is for the peer. `detail` is for the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub detail: String,
}

impl ProtocolError {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for ProtocolError {}
