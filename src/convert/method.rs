use super::IntoRequestParts;
use crate::Error;
use http::{request, Method};

impl IntoRequestParts for Method {
    fn into_request_parts(self, mut parts: request::Parts) -> Result<request::Parts, Error> {
        parts.method = self;
        Ok(parts)
    }
}
