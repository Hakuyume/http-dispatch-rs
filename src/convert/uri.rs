use super::IntoRequestParts;
use crate::Error;
use http::{request, Uri};

impl IntoRequestParts for Uri {
    fn into_request_parts(self, mut parts: request::Parts) -> Result<request::Parts, Error> {
        parts.uri = self;
        Ok(parts)
    }
}
