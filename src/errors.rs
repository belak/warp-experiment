use std::borrow::Cow;

use warp::{hyper::StatusCode, Rejection, Reply};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid auth token: {0}")]
    AuthTokenError(String),

    #[error("unauthorized")]
    Unauthorized,
}

impl warp::reject::Reject for Error {}

pub fn auth_token(inner: impl Into<String>) -> Rejection {
    warp::reject::custom(Error::AuthTokenError(inner.into()))
}

pub fn unauthorized() -> Rejection {
    warp::reject::custom(Error::Unauthorized)
}

/// An API error serializable to JSON.
#[derive(serde::Serialize)]
struct ErrorMessage<'a> {
    code: u16,
    message: Cow<'a, str>,
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Rejection> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "not found".into())
    } else if let Some(inner) = err.find::<Error>() {
        match inner {
            Error::AuthTokenError(_) => (StatusCode::BAD_REQUEST, inner.to_string().into()),
            Error::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
        }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        (StatusCode::METHOD_NOT_ALLOWED, "method not allowed".into())
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
    };

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}
