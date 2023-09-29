use super::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::FutureExt;
use http::{Request, Response};
use serde::{Deserialize, Serialize};

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
