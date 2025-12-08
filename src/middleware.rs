use axum::{
    extract::{Request},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use axum_extra::extract::CookieJar;
use crate::api::cookie_auth::validate_auth_cookie;

pub async fn auth_middleware( jar: CookieJar, request: Request, next: Next, ) -> Result<Response, StatusCode> {
    match validate_auth_cookie(&jar) {
        Ok(_session_data) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}