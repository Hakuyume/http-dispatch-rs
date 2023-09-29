use crate::{BoxBody, Error};
use bytes::{Bytes, BytesMut};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt, TryStreamExt};
use headers::{Header, HeaderMapExt};
use http::{request, response, Method, Request, Response, StatusCode, Uri};
use http_body::{Body, Empty, Full};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tower::BoxError;

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

impl IntoRequest for () {
    fn into_request(self) -> Result<Request<BoxBody>, Error> {
        Ok(Request::builder()
            .body(Empty::new().map_err(BoxError::from).boxed_unsync())
            .unwrap())
    }
}
impl FromResponse for () {
    fn from_response(_: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
        futures::future::ok(()).boxed()
    }
}

impl IntoRequest for Bytes {
    fn into_request(self) -> Result<Request<BoxBody>, Error> {
        Ok(Request::builder()
            .body(Full::new(self).map_err(BoxError::from).boxed_unsync())
            .unwrap())
    }
}
impl FromResponse for Bytes {
    fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
        let mut body = response.into_body();
        futures::stream::poll_fn(move |cx| Pin::new(&mut body).poll_data(cx))
            .try_collect()
            .map_ok(BytesMut::freeze)
            .map_err(Error::Service)
            .boxed()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Json<T>(pub T);
impl<T> IntoRequest for Json<T>
where
    T: Serialize,
{
    fn into_request(self) -> Result<Request<BoxBody>, Error> {
        serde_json::to_vec(&self.0)
            .map_err(Error::from)
            .and_then(|body| Bytes::from(body).into_request())
    }
}
impl<T> FromResponse for Json<T>
where
    T: for<'de> Deserialize<'de> + Send + 'static,
{
    fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
        Bytes::from_response(response)
            .map(|body| {
                body.and_then(|body| serde_json::from_slice(&body).map(Self).map_err(Error::from))
            })
            .boxed()
    }
}

impl IntoRequestParts for Method {
    fn into_request_parts(self, mut parts: request::Parts) -> Result<request::Parts, Error> {
        parts.method = self;
        Ok(parts)
    }
}

impl FromResponseParts for StatusCode {
    fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>> {
        futures::future::ok(parts.status).boxed()
    }
}

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

impl IntoRequestParts for Uri {
    fn into_request_parts(self, mut parts: request::Parts) -> Result<request::Parts, Error> {
        parts.uri = self;
        Ok(parts)
    }
}

macro_rules! impl_tuple {
    ($($ty:ident),*) => {
        impl<$($ty),*, R> IntoRequest for ($($ty),*, R)
        where
            $($ty: IntoRequestParts,)*
            R: IntoRequest,
        {
            #[allow(non_snake_case)]
            fn into_request(self) -> Result<Request<BoxBody>, Error> {
                let ($($ty),*, R) = self;
                let (parts, body) = R.into_request()?.into_parts();
                $(let parts = $ty.into_request_parts(parts)?;)*
                Ok(Request::from_parts(parts, body))
            }
        }

        impl<$($ty),*, R> FromResponse for ($($ty),*, R)
        where
            $($ty: FromResponseParts + Send,)*
            R: FromResponse + Send,
        {
            #[allow(non_snake_case)]
            fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
                Box::pin(async move {
                    let (mut parts, body) = response.into_parts();
                    $(let $ty = $ty::from_response_parts(&mut parts).await?;)*
                    let R = R::from_response(Response::from_parts(parts, body)).await?;
                    Ok(($($ty),*, R))
                })
            }
        }
    };
}
impl_tuple!(T1);
impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
