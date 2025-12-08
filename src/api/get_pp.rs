use axum::{
    Json, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse
};
use serde::Serialize;
use std::fs;
use std::path::Path as StdPath;
use crate::ApiState;

/**
 * Profile picture response structure with image data and headers
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @field header  HTTP headers for proper image serving
 * @field img     Optional binary image data
 * @author        Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct PpResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing)]
    header: HeaderMap,
    img: Option<Vec<u8>>,
}

/**
 * Handles profile picture retrieval requests with fallback to default image
 * 
 * @param state Shared API state containing database connection
 * @param id    User ID for which to retrieve the profile picture
 * @return      HTTP response with image data or error message
 * @author      Vander Putten Benjamin
 */
pub async fn get_pp(State(state): State<ApiState>, Path(id): Path<i64>) -> impl IntoResponse {
    // GET pp
    let user = match state.get_user_pp(id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // NO Personalized PP
            return serve_default_image().await;
        }
        Err(e) => {
            eprintln!("Get PP Error: {}", e);
            return serve_default_image().await;
        }
    };

    // Verify the path is secured and in the correct directory
    if !user.pp_link.starts_with("img/") {
        eprintln!("No Valid Path: {}", user.pp_link);
        return serve_default_image().await;
    }

    let full_path = format!("static/{}", user.pp_link);

    // Read file and detect MIME type dynamically
    match fs::read(&full_path) {
        Ok(img_data) => {
            let content_type = detect_content_type(&full_path, &img_data);
            
            let mut headers = HeaderMap::new();
            headers.insert("content-type", content_type.parse().unwrap());
            headers.insert("cache-control", "max-age=3600".parse().unwrap());
            
            let response = PpResponse {
                success: true,
                message: "Image found".to_string(),
                header: headers,
                img: Some(img_data),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            eprintln!("Error while reading picture data: {}", e);
            serve_default_image().await
        }
    }
}

/**
 * Detects the appropriate content type for an image file
 * 
 * @param file_path Path to the image file
 * @param file_data Binary data of the file for magic number detection
 * @return          MIME type string
 * @author          Vander Putten Benjamin
 */
fn detect_content_type(file_path: &str, file_data: &[u8]) -> String {
    // First try to detect from file extension
    let path = StdPath::new(file_path);
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            "png" => return "image/png".to_string(),
            "jpg" | "jpeg" => return "image/jpeg".to_string(),
            "gif" => return "image/gif".to_string(),
            "webp" => return "image/webp".to_string(),
            "bmp" => return "image/bmp".to_string(),
            "svg" => return "image/svg+xml".to_string(),
            "ico" => return "image/x-icon".to_string(),
            _ => {} // Fall through to magic number detection
        }
    }

    // Fallback to magic number detection for common formats
    if file_data.len() >= 8 {
        match &file_data[0..8] {
            [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] => return "image/png".to_string(),
            [0xFF, 0xD8, 0xFF, _] => return "image/jpeg".to_string(),
            [0x47, 0x49, 0x46, 0x38, 0x37, 0x61] | [0x47, 0x49, 0x46, 0x38, 0x39, 0x61] => {
                return "image/gif".to_string()
            }
            [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50] => {
                return "image/webp".to_string()
            }
            [0x42, 0x4D, _, _, _, _, _, _] => return "image/bmp".to_string(),
            _ => {}
        }
    }

    // Default fallback
    "application/octet-stream".to_string()
}

/**
 * Serves the default profile picture when user picture is unavailable
 * 
 * @return HTTP response with default image data or error message
 * @author Vander Putten Benjamin
 */
async fn serve_default_image() -> (StatusCode, Json<PpResponse>) {
    let default_path = "static/img/people.jpg";
    
    match fs::read(default_path) {
        Ok(img_data) => {
            let content_type = detect_content_type(default_path, &img_data);
            
            let mut headers = HeaderMap::new();
            headers.insert("content-type", content_type.parse().unwrap());
            headers.insert("cache-control", "max-age=3600".parse().unwrap());
            
            let response = PpResponse {
                success: true,
                message: "Default Picture found".to_string(),
                header: headers,
                img: Some(img_data),
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            eprintln!("Error while reading default picture data: {}", e);
            let mut headers = HeaderMap::new();
            headers.insert("content-type", "text/plain".parse().unwrap());
            let response = PpResponse {
                success: false,
                message: "Picture Not Found".to_string(),
                header: headers,
                img: None,
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}