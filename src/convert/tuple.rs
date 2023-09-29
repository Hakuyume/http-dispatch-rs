use super::{FromResponse, FromResponseParts, IntoRequest, IntoRequestParts};
use crate::{BoxBody, Error};
use futures::future::BoxFuture;
use http::{Request, Response};

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
