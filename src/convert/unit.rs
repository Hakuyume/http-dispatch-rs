use super::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use futures::future::BoxFuture;
use futures::FutureExt;
use http::{Request, Response};
use http_body::{Body, Empty};
use tower::BoxError;

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
