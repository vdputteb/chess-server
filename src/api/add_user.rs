use askama_axum::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::ApiState;
use crate::api::crypt::hash_pass;

/**
 * User registration request structure
 * 
 * @field username Desired username for the new account
 * @field email    Email address for the new account
 * @field password Plain text password for the new account
 * @author         Vander Putten Benjamin
 */
#[derive(Debug, Deserialize)]
pub struct AddRequest {
    username: String,
    email: String,
    password: String,
}

/**
 * User registration response structure
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @author        Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct AddResponse {
    success: bool,
    message: String,
}

/**
 * Handles new user registration with username availability check and password hashing
 * 
 * @param state Shared API state containing database connection
 * @param data  User registration data from client
 * @return      HTTP response indicating registration success or failure
 * @author      Vander Putten Benjamin
 */
pub async fn add_user(state: State<ApiState>, Json(data): Json<AddRequest>) -> impl IntoResponse {
    
    // Check if username already exists
    if let Ok(Some(_)) = state.get_user_by_username(&data.username).await {
        let response = AddResponse {
            success: false,
            message: "Username already exists".to_string(),
        };
        return (StatusCode::CONFLICT, Json(response));
    } 

    // Check if email already exists
    if let Ok(Some(_)) = state.get_user_by_email(&data.email).await {
        let response = AddResponse {
            success: false,
            message: "Email already exists".to_string(),
        };
        return (StatusCode::CONFLICT, Json(response));
    } 

    // Hash password for secure storage (maintenant déterministe)
    let password_hash = match hash_pass(&data.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Hashing error: {}", e);
            let response = AddResponse {
                success: false,
                message: "Server error".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Create new user in database
    match state.create_user(&data.username, &data.email, &password_hash).await {
        Ok(_user) => {
            let response = AddResponse {
                success: true,
                message: "User added successfully ".to_string(),
            };
            return (StatusCode::CREATED, Json(response))
        }
        Err(e) => {
            eprintln!("User creation error: {}", e);
            let response = AddResponse {
                success: false,
                message: "Error during creation".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}