use bytes::Bytes;
use http::StatusCode;
use tower::BoxError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Service(BoxError),
    #[error("[{status:?}] {body:?}")]
    Http { status: StatusCode, body: Bytes },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
