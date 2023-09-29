use super::FromResponseParts;
use crate::Error;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::{response, StatusCode};

impl FromResponseParts for StatusCode {
    fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>> {
        futures::future::ok(parts.status).boxed()
    }
}
