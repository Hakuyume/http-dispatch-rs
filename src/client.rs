use crate::convert::{FromResponse, IntoRequest};
use crate::{BoxBody, Error};
use bytes::Bytes;
use futures::future::{BoxFuture, Either};
use futures::{FutureExt, TryFutureExt};
use http::{Request, Response, StatusCode};
use http_body::Body;
use std::future;
use tower::{BoxError, Service, ServiceExt};

trait CloneService<R>: Service<R> {
    fn clone_box(
        &self,
    ) -> Box<
        dyn CloneService<R, Response = Self::Response, Error = Self::Error, Future = Self::Future>
            + Send
            + Sync,
    >;
}
impl<R, T> CloneService<R> for T
where
    T: Clone + Service<R> + Send + Sync + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<
        dyn CloneService<R, Response = Self::Response, Error = Self::Error, Future = Self::Future>
            + Send
            + Sync,
    > {
        Box::new(self.clone())
    }
}

pub struct Client(
    Box<
        dyn CloneService<
                Request<BoxBody>,
                Response = Response<BoxBody>,
                Error = BoxError,
                Future = BoxFuture<'static, Result<Response<BoxBody>, BoxError>>,
            > + Send
            + Sync,
    >,
);

impl Clone for Client {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

impl Client {
    pub fn new<S, T, U>(service: S) -> Self
    where
        S: Clone + Service<Request<T>, Response = Response<U>> + Send + Sync + 'static,
        BoxError: From<S::Error>,
        S::Future: Send,
        T: From<BoxBody> + 'static,
        U: Body<Data = Bytes> + Send + 'static,
        BoxError: From<U::Error>,
    {
        Self(Box::new(
            service
                .map_request(|request: Request<BoxBody>| {
                    let (parts, body) = request.into_parts();
                    Request::from_parts(parts, T::from(body))
                })
                .map_response(|response| {
                    let (parts, body) = response.into_parts();
                    Response::from_parts(parts, body.map_err(BoxError::from).boxed_unsync())
                })
                .map_err(BoxError::from)
                .map_future(FutureExt::boxed),
        ))
    }

    pub fn send<T, U>(&self, request: T) -> BoxFuture<'static, Result<U, Error>>
    where
        T: IntoRequest,
        U: FromResponse + 'static,
    {
        let service = self.0.clone_box();
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
