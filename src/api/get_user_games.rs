use axum::{ extract::{ Query, State }, http::StatusCode, Json };
use axum_extra::extract::CookieJar;
use serde::{ Deserialize, Serialize };
use crate::{ ApiState, api::{cookie_auth::validate_auth_cookie} };

/**
 * User games request structure
 */
#[derive(Debug, Deserialize)]
pub struct UserGamesRequest {
    pub user_id: i64,
}

/**
 * Game data representation
 */
#[derive(Debug, Serialize)]
pub struct GameData {
    pub id: i64,
    pub white_player_id: Option<i64>,
    pub black_player_id: Option<i64>,
    pub white_player_username: String,
    pub black_player_username: String,
    pub pgn: Option<String>,
    pub result: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_color: String, // "white" or "black"
    pub opponent_username: String,
}

/**
 * User games response structure
 */
#[derive(Debug, Serialize)]
pub struct UserGamesResponse {
    pub success: bool,
    pub message: String,
    pub games: Vec<GameData>,
}

/**
 * Handles user games retrieval requests
 */
pub async fn get_user_games(jar: CookieJar, State(state): State<ApiState>, Query(info): Query<UserGamesRequest> ) -> (StatusCode, Json<UserGamesResponse>) {
    
    let _session_data = match validate_auth_cookie(&jar) {
        Ok(data) => data,
        Err(err) => {
            let response = UserGamesResponse {
                success: false,
                message: format!("Authentification requise: {}", err),
                games: Vec::new(),
            };
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };

    // Get user games from database
    match state.get_user_games(info.user_id).await {
        Ok(games) => {
            let mut game_data_list = Vec::new();

            for game in games {
                // Déterminer la couleur de l'utilisateur
                let user_color = if game.white_player_id == Some(info.user_id) {
                    "white"
                } else if game.black_player_id == Some(info.user_id) {
                    "black"
                } else {
                    // Ce cas ne devrait pas arriver normalement
                    continue;
                };

                // Récupérer les informations des joueurs avec gestion des utilisateurs supprimés (NULL)
                let (white_player_username, black_player_username) = get_player_usernames(
                    &state,
                    game.white_player_id,
                    game.black_player_id
                ).await;

                // Déterminer le nom de l'adversaire
                let opponent_username = if user_color == "white" {
                    black_player_username.clone()
                } else {
                    white_player_username.clone()
                };

                game_data_list.push(GameData {
                    id: game.id,
                    white_player_id: game.white_player_id,
                    black_player_id: game.black_player_id,
                    white_player_username,
                    black_player_username,
                    pgn: game.pgn,
                    result: game.result,
                    created_at: game.created_at,
                    user_color: user_color.to_string(),
                    opponent_username,
                });
            }

            let response = UserGamesResponse {
                success: true,
                message: "Games retrieved successfully".to_string(),
                games: game_data_list,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response = UserGamesResponse {
                success: false,
                message: "Server error".to_string(),
                games: Vec::new(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

/**
 * Helper function to get usernames for both players with NULL handling
 */
async fn get_player_usernames( state: &ApiState, white_player_id: Option<i64>, black_player_id: Option<i64> ) -> (String, String) {
    let white_username = match white_player_id {
        Some(id) =>
            match state.get_user_info(id).await {
                Ok(Some(user)) => user.username,
                Ok(None) | Err(_) => "Joueur supprimé".to_string(),
            }
        None => "Joueur supprimé".to_string(),
    };

    let black_username = match black_player_id {
        Some(id) =>
            match state.get_user_info(id).await {
                Ok(Some(user)) => user.username,
                Ok(None) | Err(_) => "Joueur supprimé".to_string(),
            }
        None => "Joueur supprimé".to_string(),
    };

    (white_username, black_username)
}
