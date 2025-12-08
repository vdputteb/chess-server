use axum::{ 
    extract::{ Json, State }, 
    http::StatusCode, 
    response::IntoResponse 
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use crate::api::cookie_auth::{create_auth_cookie, create_logout_cookie, validate_auth_cookie};
use crate::api::crypt::verify_password;
use crate::ApiState;
use crate::database::User;

/**
 * Safe user representation without sensitive data for client responses
 * 
 * @field id       User identifier
 * @field username User's display name
 * @author         Vander Putten Benjamin
 */
#[derive(Debug, Clone, Serialize)]
pub struct SafeUser {
    id: i64,
    username: String,
}

/**
 * Converts a full User entity into a SafeUser for secure client transmission
 * 
 * @param user Complete User object from database
 * @return     SafeUser without sensitive information
 * @author     Vander Putten Benjamin
 */
impl From<User> for SafeUser {
    fn from(user: User) -> Self {
        SafeUser {
            id: user.id,
            username: user.username,
        }
    }
}

/**
 * Standardized authentication response format for API consistency
 * 
 * @field success Operation status flag
 * @field message Descriptive response message
 * @field data    Optional SafeUser data on successful authentication
 * @author        Vander Putten Benjamin
 */
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    success: bool,
    message: String,
    data: Option<SafeUser>,
}

/**
 * Login request data structure for user authentication
 * 
 * @field username User's identifier for login
 * @field password User's plain text password for verification
 * @author         Vander Putten Benjamin
 */
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

/**
 * Handles user authentication requests and returns appropriate responses with cookies
 * 
 * @param jar   CookieJar to manage cookies
 * @param state Shared database state
 * @param data  Login credentials from client
 * @return      HTTP response with authentication result and cookie
 * @author      Vander Putten Benjamin
 */
pub async fn auth( jar: CookieJar, State(state): State<ApiState>, Json(information): Json<AuthRequest> ) -> impl IntoResponse {
    
    // GET User
    let user = match state.get_user_by_username(&information.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response = AuthResponse {
                success: false,
                message: "Utilisateur non trouvé".to_string(),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
        Err(e) => {
            eprintln!("Erreur base de données: {}", e);
            let response = AuthResponse {
                success: false,
                message: "Erreur serveur".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Password Verification
    match verify_password(&information.password, &user.password) {
        Ok(true) => {
            // Mot de passe correct - continuer
        }
        Ok(false) => {
            let response = AuthResponse {
                success: false,
                message: "Mot de passe incorrect".to_string(),
                data: None,
            };
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
        Err(e) => {
            eprintln!("Erreur de vérification: {}", e);
            let response = AuthResponse {
                success: false,
                message: "Erreur serveur".to_string(),
                data: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }

    let user_data = SafeUser::from(user);

    // Créer le cookie d'authentification
    let auth_cookie = create_auth_cookie(user_data.id, &user_data.username);
    
    let jar = jar.add(auth_cookie);

    let response = AuthResponse {
        success: true,
        message: "Connexion réussie".to_string(),
        data: Some(user_data),
    };
    
    (jar, (StatusCode::OK, Json(response))).into_response()
}

/**
 * Déconnexion de l'utilisateur
 * 
 * @param jar CookieJar pour supprimer le cookie
 * @return    HTTP response avec cookie expiré
 * @author    Vander Putten Benjamin
 */
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let logout_cookie = create_logout_cookie();
    let jar = jar.add(logout_cookie);
    
    let response = AuthResponse {
        success: true,
        message: "Déconnexion réussie".to_string(),
        data: None,
    };
    
    (jar, (StatusCode::OK, Json(response))).into_response()
}

/**
 * Vérifie si l'utilisateur est authentifié
 * 
 * @param jar CookieJar contenant les cookies
 * @return    HTTP response avec statut d'authentification
 * @author    Vander Putten Benjamin
 */
pub async fn check_auth(jar: CookieJar) -> impl IntoResponse {
    match validate_auth_cookie(&jar) {
        Ok(session_data) => {
            let response = AuthResponse {
                success: true,
                message: "Authentifié".to_string(),
                data: Some(SafeUser {
                    id: session_data.user_id,
                    username: session_data.username,
                }),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(err) => {
            let response = AuthResponse {
                success: false,
                message: format!("Non authentifié: {}", err),
                data: None,
            };
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}