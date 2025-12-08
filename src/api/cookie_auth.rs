use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::api::crypt::get_jwt_secret;

// Cookie configuration
const COOKIE_NAME: &str = "session";
const COOKIE_MAX_AGE_HOURS: i64 = 1; // 1 day
const COOKIE_HTTP_ONLY: bool = true;
const COOKIE_SECURE: bool = true; // À mettre à false en développement local
const COOKIE_SAME_SITE: SameSite = SameSite::Strict;

/**
 * Structure for the session data stored in the cookie
 * @author Vander Putten Benjamin
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i64,
    pub username: String,
    pub expires: u64,
}

/**
 * Create a secure authentication cookie
 *
 * @param user_id   The user's ID
 * @param username  The username
 * @return          A secure cookie containing the session
 * @author        Vander Putten Benjamin
 */
pub fn create_auth_cookie(user_id: i64, username: &str) -> Cookie<'static> {
    let session_data = SessionData {
        user_id,
        username: username.to_string(),
        expires: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() + (COOKIE_MAX_AGE_HOURS * 3600) as u64,
    };

    // Serialize and sign the session data
    let session_json = serde_json::to_string(&session_data)
        .expect("Failed to serialize session data");
    
    let signature = sign_session_data(&session_json);
    let cookie_value = format!("{}.{}", session_json, signature);

    let mut cookie = Cookie::new(COOKIE_NAME, cookie_value);
    cookie.set_path("/");
    cookie.set_max_age(cookie::time::Duration::hours(COOKIE_MAX_AGE_HOURS));
    cookie.set_http_only(COOKIE_HTTP_ONLY);
    cookie.set_secure(COOKIE_SECURE);
    cookie.set_same_site(COOKIE_SAME_SITE);
    
    cookie
}

/**
 * Validate and extract session data from a cookie
 *
 * @param jar CookieJar containing the request's cookies
 * @return    Result<SessionData, &str> if the cookie is valid
 * @author    Vander Putten Benjamin
 */
pub fn validate_auth_cookie(jar: &CookieJar) -> Result<SessionData, &'static str> {
    let cookie = match jar.get(COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Err("No auth cookie found"),
    };

    let value = cookie.value();
    let parts: Vec<&str> = value.split('.').collect();
    
    if parts.len() != 2 {
        return Err("Invalid cookie format");
    }

    let session_json = parts[0];
    let provided_signature = parts[1];

    // Verify the signature
    let expected_signature = sign_session_data(session_json);
    if !constant_time_comparison(provided_signature, &expected_signature) {
        return Err("Invalid signature");
    }

    // Deserialize the session data
    let session_data: SessionData = serde_json::from_str(session_json)
        .map_err(|_| "Invalid session data")?;

    // Check expiration
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    if current_time > session_data.expires {
        return Err("Session expired");
    }

    Ok(session_data)
}

/**
 * Create a logout cookie (to remove the cookie client-side)
 *
 * @return Cookie for logging out
 * @author Vander Putten Benjamin
 */
pub fn create_logout_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::new(COOKIE_NAME, "");
    cookie.set_path("/");
    cookie.set_max_age(cookie::time::Duration::seconds(0)); // Expire immédiatement
    cookie.set_http_only(COOKIE_HTTP_ONLY);
    cookie.set_secure(COOKIE_SECURE);
    cookie.set_same_site(COOKIE_SAME_SITE);
    
    cookie
}

/**
 * Sign the session data
 *
 * @param data Data to sign
 * @return     Hexadecimal signature
 * @author     Vander Putten Benjamin
 */
fn sign_session_data(data: &str) -> String {
    let secret = get_jwt_secret();
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/**
 * Constant-time comparison to avoid timing attacks
 *
 * @param a First string to compare
 * @param b Second string to compare
 * @return  true if the strings are identical
 * @author  Vander Putten Benjamin
 */
fn constant_time_comparison(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    a.bytes()
        .zip(b.bytes())
        .fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}