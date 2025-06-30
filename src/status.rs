/// HTTP status codes for responses.
pub enum StatusCode {
    Ok,
    NotFound,
    BadRequest,
    InternalServerError,
}

impl StatusCode {
    pub fn as_str(&self) -> &str {
        match self {
            StatusCode::Ok => "200 OK",
            StatusCode::NotFound => "404 NOT FOUND",
            StatusCode::BadRequest => "400 BAD REQUEST",
            StatusCode::InternalServerError => "500 INTERNAL SERVER ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_as_str() {
        assert_eq!(StatusCode::Ok.as_str(), "200 OK");
        assert_eq!(StatusCode::NotFound.as_str(), "404 NOT FOUND");
        assert_eq!(StatusCode::BadRequest.as_str(), "400 BAD REQUEST");
        assert_eq!(
            StatusCode::InternalServerError.as_str(),
            "500 INTERNAL SERVER ERROR"
        );
    }
}
