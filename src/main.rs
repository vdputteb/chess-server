mod database;
mod api;
mod templates;
mod middleware;

use axum::{
    Router, extract::State, routing::{get, post, put}
};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use dotenvy::dotenv;
use std::{env, sync::Arc};
use crate::{ database::Database};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tower_cookies::{CookieManagerLayer};
// Import des handlers de templates
use templates::{serve_account,serve_login, serve_online, serve_param, serve_solo};
use axum::middleware::from_fn;
use crate::middleware::auth_middleware;

// ---------------------------
// Types partagés pour WebSocket
// ---------------------------

type Tx = mpsc::UnboundedSender<Message>;
type Rooms = Arc<DashMap<String, Vec<Tx>>>;
pub type ApiState = Arc<Database>;

// Message reçu du client
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMsg {
    Join { room: String, name: String },
    Move { room: String, from: String, to: String, promotion: Option<char> },
    Chat { room: String, message: String },
}

// Message envoyé au client
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum ServerMsg {
    Joined { name: String },
    Moved { from: String, to: String, promotion: Option<char> },
    Chatted { name: String, message: String },
    Error { message: String },
}

// ---------------------------
// Point d'entrée
// ---------------------------


pub fn init() {
    dotenv().ok();
}


#[tokio::main]
async fn main() {
    // Initialiser la base de données
    init();
    let db_url = match env::var("DATABASE_URL") {
        Ok(db_url) => db_url,
        Err(e) => {
            eprintln!("DB Url error: {}", e);
            return
        }
    };

    let database = match database::Database::new(&db_url).await {
        Ok(db) => {
            println!("Base de données connectée avec succès");
            db
        }
        Err(e) => {
            eprintln!("Erreur connexion base de données: {}", e);
            std::process::exit(1);
        }
    };

    let rooms: Rooms = Arc::new(DashMap::new());
    let api_state: ApiState = Arc::new(database);

    // Routeur pour l'API (avec état Database)
    let api_router = Router::new()
        .route("/auth/register", post(api::add_user::add_user))
        .route("/auth/login", post(api::auth::auth))
        .route("/auth/logout", post(api::auth::logout)) 
        .route("/auth/check", get(api::auth::check_auth)) 
        .route("/auth/check-username", get(api::check_username::check_username))
        .route("/users/profile-update", post(api::upload_pp::upload_pp))
        .route("/users/profile-picture/:id", get(api::get_pp::get_pp))  
        .route("/users/profile-pp", put(api::update_user::update_user))
        .route("/users/data", post(api::get_user_datas::get_user_data))
        .route("/users/games", get(api::get_user_games::get_user_games))
        .with_state(api_state.clone());
    
    // Routeur pour les pages et WebSocket (avec état Rooms)
    let protected_router = Router::new()
        // Routes WebSocket
        .route("/ws", get(ws_handler))

        // Routes pour les pages
        .route("/", get(serve_account))
        .route("/solo", get(serve_solo))
        .route("/online", get(serve_online))
        .route("/param", get(serve_param))
        
        .layer(from_fn(auth_middleware))
        .with_state(rooms.clone());

    let public_router = Router::new()
        // Routes pour les pages
        .route("/login", get(serve_login))

        // Routes pour les fichiers statiques
        .route("/static/js/:file", get(serve_js))
        .route("/static/style/:file", get(serve_css))
        .route("/static/img/:file", get(serve_image))
        .with_state(rooms.clone());

    // Combiner les routeurs
    let app = Router::new()
        .nest("/api", api_router)
        .merge(protected_router)
        .merge(public_router)
        .layer(CookieManagerLayer::new());


    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
    println!("Serveur WebSocket en écoute sur : http://{}", addr);
    println!("Interface web disponible sur : http://{}", addr);
    println!("Page de login sur : http://{}/login", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ---------------------------
// Handlers pour servir les fichiers statiques
// ---------------------------

async fn serve_css(axum::extract::Path(file): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    let css_content = match file.as_str() {
        "base.css" => include_str!("../static/style/base.css"),
        "login.css" => include_str!("../static/style/login.css"),
        "account.css" => include_str!("../static/style/account.css"),
        _ => return axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Fichier CSS non trouvé"))
            .unwrap(),
    };

    axum::response::Response::builder()
        .header("content-type", "text/css")
        .body(axum::body::Body::from(css_content))
        .unwrap()
}

async fn serve_js(axum::extract::Path(file): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    let js_content = match file.as_str() {
        "login.js" => include_str!("../static/js/login.js"),
        "client.js" => include_str!("../static/js/client.js"),
        "base.js" => include_str!("../static/js/base.js"),
        "account.js" => include_str!("../static/js/account.js"),
        "add.js" => include_str!("../static/js/add.js"),
        "toggleForms.js" => include_str!("../static/js/toggleForms.js"),
        _ => return axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Fichier JS non trouvé"))
            .unwrap(),
    };

    axum::response::Response::builder()
        .header("content-type", "application/javascript")
        .body(axum::body::Body::from(js_content))
        .unwrap()
}

async fn serve_image(axum::extract::Path(file): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    let (content, content_type) = match file.as_str() {
        "logo.png" => (include_bytes!("../static/img/logo.png").as_slice(), "image/png"),
        "ChessPieces.png" => (include_bytes!("../static/img/ChessPieces.png").as_slice(), "image/png"),
        "ImgLeft.png" => (include_bytes!("../static/img/ImgLeft.png").as_slice(), "image/png"),
        "ImgRight.png" => (include_bytes!("../static/img/ImgRight.png").as_slice(), "image/png"),
        "log-out.png" => (include_bytes!("../static/img/log-out.png").as_slice(), "image/png"),
        "people.jpg" => (include_bytes!("../static/img/people.jpg").as_slice(), "image/jpeg"),
        "settings.png" => (include_bytes!("../static/img/settings.png").as_slice(), "image/png"),
        _ => return axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("Image non trouvée"))
            .unwrap(),
    };

    axum::response::Response::builder()
        .header("content-type", content_type)
        .body(axum::body::Body::from(content.to_vec()))
        .unwrap()
}

// ---------------------------
// Handler WebSocket
// ---------------------------

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(rooms): State<Rooms>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| websocket_loop(socket, rooms))
}

// ---------------------------
// Boucle WebSocket
// ---------------------------

async fn websocket_loop(ws: WebSocket, rooms: Rooms) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let tx_clone = tx.clone();

    let (mut sender, mut receiver) = ws.split();

    // Tâche qui envoie les messages vers le client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut joined_room: Option<String> = None;
    let mut user_name: Option<String> = None;

    // Lire les messages entrants
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                if let Ok(cmd) = serde_json::from_str::<ClientMsg>(&text) {
                    match cmd {
                        ClientMsg::Join { room, name } => {
                            // Retirer des rooms précédents
                            if let Some(old_room) = &joined_room {
                                if let Some(mut list) = rooms.get_mut(old_room) {
                                    list.retain(|client_tx| !client_tx.same_channel(&tx_clone));
                                }
                            }
                            
                            // Ajouter au nouveau room
                            rooms.entry(room.clone())
                                .or_default()
                                .push(tx_clone.clone());
                            joined_room = Some(room.clone());
                            user_name = Some(name.clone());
                            
                            // Notifier les autres clients
                            if let Some(clients) = rooms.get(&room) {
                                let join_msg = ServerMsg::Joined { name: name.clone() };
                                if let Ok(join_msg_json) = serde_json::to_string(&join_msg) {
                                    for client in clients.iter() {
                                        if !client.same_channel(&tx_clone) {
                                            let _ = client.send(Message::Text(join_msg_json.clone()));
                                        }
                                    }
                                }
                            }
                            
                            // Confirmer la connexion au client
                            let welcome_msg = ServerMsg::Chatted { 
                                name: "Serveur".to_string(), 
                                message: format!("Bienvenue dans la room '{}', {}!", room, name)
                            };
                            if let Ok(welcome_json) = serde_json::to_string(&welcome_msg) {
                                let _ = tx.send(Message::Text(welcome_json));
                            }
                        }

                        ClientMsg::Move { room, from, to, promotion } => {
                            if let Some(clients) = rooms.get(&room) {
                                let move_msg = ServerMsg::Moved { from, to, promotion };
                                if let Ok(move_msg_json) = serde_json::to_string(&move_msg) {
                                    for client in clients.iter() {
                                        let _ = client.send(Message::Text(move_msg_json.clone()));
                                    }
                                }
                            }
                        }

                        ClientMsg::Chat { room, message } => {
                            if let (Some(clients), Some(name)) = (rooms.get(&room), &user_name) {
                                let chat_msg = ServerMsg::Chatted { 
                                    name: name.clone(), 
                                    message 
                                };
                                if let Ok(chat_msg_json) = serde_json::to_string(&chat_msg) {
                                    for client in clients.iter() {
                                        let _ = client.send(Message::Text(chat_msg_json.clone()));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Envoyer une erreur si le JSON est invalide
                    let error_msg = ServerMsg::Error { 
                        message: "Message JSON invalide".to_string() 
                    };
                    if let Ok(error_json) = serde_json::to_string(&error_msg) {
                        let _ = tx.send(Message::Text(error_json));
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(_) => {
                break;
            }
            _ => {}
        }
    }

    // Déconnexion : retirer le client du room
    if let Some(room) = &joined_room {
        if let Some(mut list) = rooms.get_mut(room) {
            list.retain(|client_tx| !client_tx.same_channel(&tx_clone));
            
            // Notifier les autres joueurs de la déconnexion
            if let Some(name) = &user_name {
                let leave_msg = ServerMsg::Chatted {
                    name: "Serveur".to_string(),
                    message: format!("{} a quitté la partie", name)
                };
                if let Ok(leave_json) = serde_json::to_string(&leave_msg) {
                    for client in list.iter() {
                        if !client.same_channel(&tx_clone) {
                            let _ = client.send(Message::Text(leave_json.clone()));
                        }
                    }
                }
            }
        }
    }

    send_task.abort();
}