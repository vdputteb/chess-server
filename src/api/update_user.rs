use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{ApiState, api::{cookie_auth::validate_auth_cookie, crypt::{hash_pass}}};

/**
 * User update request structure containing all fields for profile modification
 * 
 * @field id           User ID to update
 * @field username     New username value
 * @field email        New email address
 * @field old_password Current password for verification
 * @field new_password New password to set
 * @field token        Authentication token for authorization
 * @author            Vander Putten Benjamin
 */
#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    id: i64,
    username: String,
    email: String,
    old_password: String,
    new_password: String,
}

/**
 * Standardized response for user update operations
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @author        Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    success: bool,
    message: String,
}

/**
 * Handles user profile update requests with comprehensive security verification
 * 
 * @param state Shared API state containing database connection
 * @param data  Update request with user credentials and new data
 * @return      HTTP response indicating update success or failure
 * @author      Vander Putten Benjamin
 */
pub async fn update_user(jar: CookieJar, state: State<ApiState>, Json(data): Json<UpdateRequest>) -> (StatusCode, Json<UpdateResponse>) {
    // Token verification
    let _session_data = match validate_auth_cookie(&jar) {
        Ok(data) => data,
        Err(err) => {
            let response = UpdateResponse {
                success: false,
                message: format!("Authentification requise: {}", err),
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // User existence verification
    let user = match state.get_user_by_username(&data.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response = UpdateResponse {
                success: false,
                message: "User not found".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response = UpdateResponse {
                success: false,
                message: "Server error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Current password verification - Hash error handling
    let hash_password = match hash_pass(&data.old_password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Hashing error: {}", e);
            let response = UpdateResponse {
                success: false,
                message: "Server error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };
    
    if user.password != hash_password {
        let response = UpdateResponse {
            success: false,
            message: "Incorrect password".to_string(),
        };
        return (StatusCode::UNAUTHORIZED, Json(response));
    }

    // Hash new password and update user
    let new_password_hash = match hash_pass(&data.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Hashing error: {}", e);
            let response = UpdateResponse {
                success: false,
                message: "Server error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Correction: use new_password_hash instead of hash_password for the new password
    match state.update_user(data.id, &data.username, &data.email, &hash_password, &new_password_hash)
    .await {
        Ok(Some(_user)) => {
            let response = UpdateResponse {
                success: true,
                message: "Profile updated successfully".to_string(),
            };
            (StatusCode::OK, Json(response))
        }
        Ok(None) => {
            let response = UpdateResponse {
                success: false,
                message: "Profile not updated".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response = UpdateResponse {
                success: false,
                message: "Server error".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}