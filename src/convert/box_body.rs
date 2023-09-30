use super::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use futures::future::BoxFuture;
use futures::FutureExt;
use http::{Request, Response};

impl IntoRequest for BoxBody {
    fn into_request(self) -> Result<Request<BoxBody>, Error> {
        Ok(Request::builder().body(self).unwrap())
    }
}

impl FromResponse for BoxBody {
    fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
        futures::future::ok(response.into_body()).boxed()
    }
}
