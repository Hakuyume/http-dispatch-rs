use super::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use bytes::{Bytes, BytesMut};
use futures::future::BoxFuture;
use futures::{FutureExt, TryFutureExt, TryStreamExt};
use http::{Request, Response};
use http_body::{Body, Full};
use std::pin::Pin;
use tower::BoxError;

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
