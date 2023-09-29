use super::{FromResponseParts, IntoRequestParts};
use crate::Error;
use futures::future::BoxFuture;
use futures::FutureExt;
use headers::{Header, HeaderMapExt};
use http::{request, response};

#[derive(Clone, Copy, Debug)]
pub struct TypedHeader<T>(pub T);
impl<T> IntoRequestParts for TypedHeader<T>
where
    T: Header,
{
    fn into_request_parts(self, mut parts: request::Parts) -> Result<request::Parts, Error> {
        parts.headers.typed_insert(self.0);
        Ok(parts)
    }
}
impl<T> FromResponseParts for Option<TypedHeader<T>>
where
    T: Header + Send + 'static,
{
    fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>> {
        futures::future::ok(parts.headers.typed_get().map(TypedHeader)).boxed()
    }
}
