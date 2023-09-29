mod client;
mod convert;
mod error;
#[cfg(test)]
mod tests;

use bytes::Bytes;
pub use client::Client;
pub use convert::{Json, TypedHeader};
pub use error::Error;
pub use headers;
pub use http;
use http_body::combinators::UnsyncBoxBody;
use tower::BoxError;

pub type BoxBody = UnsyncBoxBody<Bytes, BoxError>;
