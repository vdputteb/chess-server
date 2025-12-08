// api/check_username.rs
use askama_axum::IntoResponse;
use axum::{Json, extract::{State, Query}, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::ApiState;

/**
 * Request structure for checking username availability
 * 
 * @field username The username to check
 * @author Vander Putten Benjamin
 */
#[derive(Debug, Deserialize)]
pub struct CheckUsernameRequest {
    username: String,
}

/**
 * Response structure for username verification
 * 
 * @field available Indicates if the username is available
 * @field message   Descriptive message
 * @author Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    message: String,
}

/**
 * Checks the availability of a username
 * 
 * @param state Shared state containing the database connection
 * @param query Query parameters containing the username to check
 * @return      HTTP response indicating username availability
 * @author      Vander Putten Benjamin
 */
pub async fn check_username(state: State<ApiState>, Query(query): Query<CheckUsernameRequest>) -> impl IntoResponse {
    let username = query.username.trim();
    
    // Basic validation
    if username.is_empty() {
        let response = CheckUsernameResponse {
            available: false,
            message: "Username cannot be empty".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    
    if username.len() < 3 {
        let response = CheckUsernameResponse {
            available: false,
            message: "Username must contain at least 3 characters".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    
    if username.len() > 50 {
        let response = CheckUsernameResponse {
            available: false,
            message: "Username cannot exceed 50 characters".to_string(),
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Check if username already exists
    match state.get_user_by_username(username).await {
        Ok(Some(_)) => {
            let response = CheckUsernameResponse {
                available: false,
                message: "This username is already taken".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = CheckUsernameResponse {
                available: true,
                message: "This username is available".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            eprintln!("Error while checking username: {}", e);
            let response = CheckUsernameResponse {
                available: false,
                message: "Server error during verification".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}
