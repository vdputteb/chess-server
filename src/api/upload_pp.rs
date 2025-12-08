use axum::{extract::{Json, Multipart, State}, http::StatusCode};
use serde::Serialize;
use tokio::fs as tokio_fs;
use crate::ApiState;

/**
 * Response structure for profile picture upload operations
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @field link    Optional link to the uploaded profile picture
 * @author        Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct UploadPpResponse {
    success: bool,
    message: String,
    link: Option<String>,
}

/**
 * Handles multipart form uploads for user profile pictures with validation
 * 
 * @param state     Shared API state containing database connection
 * @param multipart Multipart form data containing username and image file
 * @return          HTTP response with upload result and profile picture link
 * @author          Vander Putten Benjamin
 */
pub async fn upload_pp(state: State<ApiState>, mut multipart: Multipart) -> (StatusCode, Json<UploadPpResponse>) {
    let mut username = None;
    let mut img_data = None;
    let mut content_type = None;

    // Extract data from multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("Error extracting multipart field: {}", e);
        let response = UploadPpResponse {
            success: false,
            message: "Error processing form".to_string(),
            link: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }).unwrap() {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "username" => {
                username = match field.text().await {
                    Ok(text) => Some(text),
                    Err(e) => {
                        eprintln!("Error reading username: {}", e);
                        let response = UploadPpResponse {
                            success: false,
                            message: "Error reading username".to_string(),
                            link: None,
                        };
                        return (StatusCode::BAD_REQUEST, Json(response));
                    }
                };
            }
            "image" => {
                content_type = field.content_type().map(|s| s.to_string());
                img_data = match field.bytes().await {
                    Ok(bytes) => Some(bytes),
                    Err(e) => {
                        eprintln!("Error reading image: {}", e);
                        let response = UploadPpResponse {
                            success: false,
                            message: "Error reading image".to_string(),
                            link: None,
                        };
                        return (StatusCode::BAD_REQUEST, Json(response));
                    }
                };
            }
            _ => {}
        }
    }

    let username = match username {
        Some(username) => username,
        None => {
            let response = UploadPpResponse {
                success: false,
                message: "Username missing".to_string(),
                link: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    };

    let img_data = match img_data {
        Some(img_data) => img_data,
        None => {
            let response = UploadPpResponse {
                success: false,
                message: "Image missing".to_string(),
                link: None,
            };
            return (StatusCode::BAD_REQUEST, Json(response));
        }
    };

    // Validate image size (5MB limit)
    if img_data.len() > 5 * 1024 * 1024 {
        let response = UploadPpResponse {
            success: false,
            message: "Image must not exceed 5MB".to_string(),
            link: None,
        };
        return (StatusCode::BAD_REQUEST, Json(response));
    }

    // Get user from database
    let user = match state.get_user_by_username(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response = UploadPpResponse {
                success: false,
                message: "User not found".to_string(),
                link: None,
            };
            return (StatusCode::NOT_FOUND, Json(response));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response = UploadPpResponse {
                success: false,
                message: "Server error".to_string(),
                link: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
        }
    };

    // Create user directory
    let user_dir = format!("static/img/{}", username);
    if let Err(e) = tokio_fs::create_dir_all(&user_dir).await {
        eprintln!("Error creating user directory: {}", e);
        let response = UploadPpResponse {
            success: false,
            message: "Error creating user directory".to_string(),
            link: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    // Determine file extension
    let extension = match content_type.as_deref() {
        Some("image/jpeg") => "jpg",
        Some("image/png") => "png",
        Some("image/gif") => "gif",
        Some("image/webp") => "webp",
        _ => "jpg", // default extension
    };

    let filename = format!("profile.{}", extension);
    let filepath = format!("{}/{}", user_dir, filename);
    let pp_link = format!("img/{}/{}", username, filename);

    // Remove old profile pictures
    let old_files = ["profile.jpg", "profile.png", "profile.gif", "profile.webp"];
    for old_file in old_files.iter() {
        let old_path = format!("{}/{}", user_dir, old_file);
        let _ = tokio_fs::remove_file(&old_path).await;
    }

    // Save new image
    if let Err(e) = tokio_fs::write(&filepath, &img_data).await {
        eprintln!("Error saving image: {}", e);
        let response = UploadPpResponse {
            success: false,
            message: "Error saving image".to_string(),
            link: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    // Update database
    if let Err(e) = state.update_pp(user.id, &pp_link).await {
        eprintln!("Error updating database: {}", e);
        // Clean up: remove the saved file if database update fails
        let _ = tokio_fs::remove_file(&filepath).await;
        let response = UploadPpResponse {
            success: false,
            message: "Error updating profile".to_string(),
            link: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response));
    }

    let response = UploadPpResponse {
        success: true,
        message: "Profile picture updated successfully".to_string(),
        link: Some(pp_link),
    };
    (StatusCode::OK, Json(response))
}