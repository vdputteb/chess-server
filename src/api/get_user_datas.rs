use askama_axum::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::{ApiState, api::cookie_auth::validate_auth_cookie, database::User };

/**
 * User data request structure with authentication
 * 
 * @field id       User ID to retrieve data for
 * @field username Username for verification
 * @field token    Authentication token for authorization
 * @author         Vander Putten Benjamin
 */
#[derive(Debug, Deserialize)]
pub struct UserRequest {
    id : i64,
    username : String,
}

/**
 * Safe user data representation without sensitive information
 * 
 * @field id         User identifier
 * @field username   User's display name
 * @field email      User's email address
 * @field created_at Account creation timestamp
 * @author           Vander Putten Benjamin
 */
#[derive(Debug,Serialize)]
pub struct UserData {
    id: i64,
    username: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

/**
 * Converts a full User entity into a safe UserData representation
 * 
 * @param user Complete User object from database
 * @return     UserData without sensitive information
 * @author     Vander Putten Benjamin
 */
impl From<User> for UserData {
    fn from(user: User) -> Self {
        UserData {
            id: user.id,
            username: user.username.clone(),
            email : user.email,
            created_at: user.created_at,
        }
    }
}

/**
 * User data retrieval response structure
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @field data    Optional UserData on successful retrieval
 * @author        Vander Putten Benjamin
 */
#[derive(Debug,Serialize)]
pub struct UserResponse {
    success : bool,
    message : String,
    data : Option<UserData>,
}

/**
 * Handles user data retrieval requests with token authentication
 * 
 * @param state Shared API state containing database connection
 * @param info  User request with ID and authentication data
 * @return      HTTP response with user data or error message
 * @author      Vander Putten Benjamin
 */
pub async fn get_user_data(jar: CookieJar, state : State<ApiState>, Json(info) : Json<UserRequest>) -> impl IntoResponse {
    let _session_data = match validate_auth_cookie(&jar) {
        Ok(data) => data,
        Err(err) => {
            let response = UserResponse {
                success: false,
                message: format!("Authentification requise: {}", err),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    match state.get_user(info.id, &info.username).await {
        Ok(Some(user)) => {
            let user_data = UserData::from(user);
            let response = UserResponse {
                success : true,
                message: "User data successful".to_string(),
                data : Some(user_data),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = UserResponse {
                success: false,
                message: "User not found".to_string(),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response = UserResponse {
                success: false,
                message: "Server error".to_string(),
                data : None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}