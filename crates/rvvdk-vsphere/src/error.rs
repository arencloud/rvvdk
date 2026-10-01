use serde::Serialize;

/// Closed diagnostic vocabulary: never retains URLs, XML, cookies or server text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("invalid connection policy or input")]
    InvalidInput,
    #[error("transport or TLS verification failed")]
    Transport,
    #[error("operation deadline exceeded")]
    Deadline,
    #[error("unexpected HTTP status or encoding")]
    Http,
    #[error("response size limit exceeded")]
    ResponseLimit,
    #[error("invalid or over-budget XML")]
    Xml,
    #[error("unexpected SOAP shape or property type")]
    Schema,
    #[error("server family or API version is not qualified")]
    UnsupportedServer,
    #[error("authentication rejected")]
    InvalidLogin,
    #[error("session not authenticated")]
    NotAuthenticated,
    #[error("permission denied")]
    NoPermission,
    #[error("server operation unsupported or restricted")]
    Restricted,
    #[error("server SOAP fault")]
    SoapFault,
    #[error("session cookie absent or invalid")]
    Cookie,
    #[error("requested property unavailable")]
    MissingProperty,
    #[error("inventory budget exceeded")]
    InventoryLimit,
    #[error("unexpected continuation page; cancellation attempted")]
    Pagination,
}

pub type Result<T> = std::result::Result<T, Error>;
