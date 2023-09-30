use crate::convert::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use bytes::Bytes;
use futures::future::{BoxFuture, Either};
use futures::{FutureExt, TryFutureExt};
use http::{Request, Response, StatusCode};
use http_body::Body;
use std::future;
use tower::util::BoxCloneService;
use tower::{BoxError, Service, ServiceExt};

#[derive(Clone)]
pub struct Client(BoxCloneService<Request<BoxBody>, Response<BoxBody>, BoxError>);

impl Client {
    pub fn new<S, T, U>(service: S) -> Self
    where
        S: Clone + Service<Request<T>, Response = Response<U>> + Send + 'static,
        BoxError: From<S::Error>,
        S::Future: Send,
        T: From<BoxBody> + 'static,
        U: Body<Data = Bytes> + Send + 'static,
        BoxError: From<U::Error>,
    {
        Self(BoxCloneService::new(
            service
                .map_request(|request: Request<BoxBody>| {
                    let (parts, body) = request.into_parts();
                    Request::from_parts(parts, T::from(body))
                })
                .map_response(|response| {
                    let (parts, body) = response.into_parts();
                    Response::from_parts(parts, body.map_err(BoxError::from).boxed_unsync())
                })
                .map_err(BoxError::from),
        ))
    }

    pub fn send<T, U>(&self, request: T) -> BoxFuture<'static, Result<U, Error>>
    where
        T: IntoRequest,
        U: FromResponse + 'static,
    {
        let service = self.0.clone();
        future::ready(request.into_request())
            .and_then(move |request| service.oneshot(request).map_err(Error::Service))
            .and_then(|response| {
                if response.status().is_success() {
                    Either::Left(U::from_response(response))
                } else {
                    Either::Right(
                        <(StatusCode, Bytes)>::from_response(response).map(
                            |response| match response {
                                Ok((status, body)) => Err(Error::Http { status, body }),
                                Err(e) => Err(e),
                            },
                        ),
                    )
                }
            })
            .boxed()
    }

    #[cfg(feature = "hyper")]
    pub fn hyper() -> Self {
        Self::new(
            hyper::Client::builder().build::<_, BoxBody>(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_webpki_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
        )
    }
}
