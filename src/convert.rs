mod bytes;
mod json;
mod method;
mod status_code;
mod tuple;
mod typed_header;
mod unit;
mod uri;

use crate::{BoxBody, Error};
use futures::future::BoxFuture;
use http::{request, response, Request, Response};
pub use json::Json;
pub use typed_header::TypedHeader;

pub trait IntoRequest {
    fn into_request(self) -> Result<Request<BoxBody>, Error>;
}

pub trait IntoRequestParts {
    fn into_request_parts(self, parts: request::Parts) -> Result<request::Parts, Error>;
}

pub trait FromResponse: Sized {
    fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>>;
}

pub trait FromResponseParts: Sized {
    fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>>;
}
