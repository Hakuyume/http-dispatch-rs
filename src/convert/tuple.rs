use super::{FromResponse, FromResponseParts, IntoRequest, IntoRequestParts};
use crate::{BoxBody, Error};
use futures::future::{BoxFuture, FutureExt, TryFutureExt};
use http::{request, response, Request, Response};

impl<T, U> IntoRequest for (T, U)
where
    T: IntoRequestParts,
    U: IntoRequest,
{
    fn into_request(self) -> Result<Request<BoxBody>, Error> {
        let (parts, body) = self.1.into_request()?.into_parts();
        Ok(Request::from_parts(self.0.into_request_parts(parts)?, body))
    }
}

impl<T, U> IntoRequestParts for (T, U)
where
    T: IntoRequestParts,
    U: IntoRequestParts,
{
    fn into_request_parts(self, parts: request::Parts) -> Result<request::Parts, Error> {
        self.1.into_request_parts(self.0.into_request_parts(parts)?)
    }
}

impl<T, U> FromResponse for (T, U)
where
    T: FromResponseParts + Send,
    U: FromResponse + Send,
{
    fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
        Box::pin(async move {
            let (mut parts, body) = response.into_parts();
            Ok((
                T::from_response_parts(&mut parts).await?,
                U::from_response(Response::from_parts(parts, body)).await?,
            ))
        })
    }
}

impl<T, U> FromResponseParts for (T, U)
where
    T: FromResponseParts + Send,
    U: FromResponseParts + Send,
{
    fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>> {
        Box::pin(async move {
            Ok((
                T::from_response_parts(parts).await?,
                U::from_response_parts(parts).await?,
            ))
        })
    }
}

macro_rules! impl_tuple {
    ([$t:ident $($ts:ident)*], [$u:ident]) => {
        impl_tuple!($t $($ts)*);
        impl_tuple!($t $($ts)* $u);
    };
    ([$t:ident $($ts:ident)*], [$u:ident $($us:ident)*]) => {
        impl_tuple!($t $($ts)*);
        impl_tuple!([$t $($ts)* $u], [$($us)*]);
    };
    ($t:ident $($ts:ident)*) => {
        impl<$t, $($ts),*> IntoRequest for ($t, $($ts),*)
        where
            ($t, ($($ts),*)): IntoRequest,
        {
            fn into_request(self) -> Result<Request<BoxBody>, Error> {
                #[allow(non_snake_case)]
                let ($t, $($ts),*) = self;
                IntoRequest::into_request(($t, ($($ts),*)))
            }
        }

        impl<$t, $($ts),*> IntoRequestParts for ($t, $($ts),*)
        where
            ($t, ($($ts),*)): IntoRequestParts,
        {
            fn into_request_parts(self, parts: request::Parts) -> Result<request::Parts, Error> {
                #[allow(non_snake_case)]
                let ($t, $($ts),*) = self;
                IntoRequestParts::into_request_parts(($t, ($($ts),*)), parts)
            }
        }

        impl<$t, $($ts),*> FromResponse for ($t, $($ts),*)
        where
            ($t, ($($ts),*)): FromResponse + 'static,
        {
            fn from_response(response: Response<BoxBody>) -> BoxFuture<'static, Result<Self, Error>> {
                #[allow(non_snake_case)]
                <($t, ($($ts),*))>::from_response(response).map_ok(|($t, ($($ts),*))| ($t, $($ts),*)).boxed()
            }
        }

        impl<$t, $($ts),*> FromResponseParts for ($t, $($ts),*)
        where
            ($t, ($($ts),*)): FromResponseParts + 'static,
        {
            fn from_response_parts(parts: &mut response::Parts) -> BoxFuture<'_, Result<Self, Error>> {
                #[allow(non_snake_case)]
                <($t, ($($ts),*))>::from_response_parts(parts).map_ok(|($t, ($($ts),*))| ($t, $($ts),*)).boxed()
            }
        }
    };
}
impl_tuple!([T1 T2 T3], [T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16]);
