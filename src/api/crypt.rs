use std::env;
/*
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
*/
use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead}};
use aes_gcm::aead::generic_array::GenericArray;

// Configuration
/*
const TOKEN_EXPIRY_SECONDS: u64 = 3600; // 1 heure

// Gestion sécurisée des tokens JWT-like

pub fn create_token(username: &str, user_id: i64) -> String {
    let secret = get_jwt_secret();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    let payload = format!("{}:{}:{}", username, user_id, timestamp);
    
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    
    format!("{}.{}", payload, hex::encode(code_bytes))
}

pub fn validate_token(token: &str, expected_username: &str) -> Result<i64, &'static str> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return Err("Invalid token format");
    }
    
    let payload = parts[0];
    let provided_signature = parts[1];
    
    // Vérifier la signature
    let secret = get_jwt_secret();
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    
    let expected_signature = hex::encode(mac.finalize().into_bytes());
    
    // Comparaison sécurisée des signatures
    if !constant_time_comparison(provided_signature, &expected_signature) {
        return Err("Invalid signature");
    }
    
    // Vérifier le payload
    let payload_parts: Vec<&str> = payload.split(':').collect();
    if payload_parts.len() != 3 {
        return Err("Invalid payload format");
    }
    
    let username = payload_parts[0];
    let user_id: i64 = payload_parts[1].parse().map_err(|_| "Invalid user ID")?;
    let timestamp: u64 = payload_parts[2].parse().map_err(|_| "Invalid timestamp")?;
    
    // Vérifier l'expiration
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    if current_time - timestamp > TOKEN_EXPIRY_SECONDS {
        return Err("Token expired");
    }
    
    // Vérifier le nom d'utilisateur
    if username != expected_username {
        return Err("Username mismatch");
    }
    
    Ok(user_id)
}

// Fonction decode_token pour compatibilité avec le code existant
pub fn decode_token(token: &str, expected_username: &str) -> bool {
    validate_token(token, expected_username).is_ok()
}

*/

// Chiffrement DÉTERMINISTE avec nonce fixe
pub fn hash_pass(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encryption_key = get_encryption_key();
    let key_bytes = hex::decode(&encryption_key)?;
    
    if key_bytes.len() != 32 {
        return Err("Encryption key must be 32 bytes".into());
    }
    
    let key = GenericArray::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    // NONCE FIXE pour avoir le même résultat à chaque fois
    let nonce = get_fixed_nonce();
    
    let ciphertext = cipher
        .encrypt(&nonce, password.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(hex::encode(result))
}

// Nonce fixe pour le hachage des mots de passe
fn get_fixed_nonce() -> GenericArray<u8, aes_gcm::aead::consts::U12> {
    *GenericArray::from_slice(b"fixed_nonce_") // 12 bytes exactement
}

// Fonction pour vérifier le mot de passe
pub fn verify_password(password: &str, encrypted_password: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let hash_result = hash_pass(password)?;
    Ok(constant_time_comparison(&hash_result, encrypted_password))
}

// Fonctions utilitaires privées
fn get_encryption_key() -> String {
    env::var("ENCRYPTION_KEY")
        .expect("ENCRYPTION_KEY must be set in .env file")
}

pub fn get_jwt_secret() -> String {
    env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string())
}

// Comparaison en temps constant pour éviter les attaques par timing
fn constant_time_comparison(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    a.bytes()
        .zip(b.bytes())
        .fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}



/*
pub fn decrypt_password(encrypted_password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let encryption_key = get_encryption_key();
    let key_bytes = hex::decode(&encryption_key)?;
    
    if key_bytes.len() != 32 {
        return Err("Encryption key must be 32 bytes".into());
    }
    
    let data = hex::decode(encrypted_password)?;
    if data.len() < 12 {
        return Err("Invalid encrypted data".into());
    }
    
    let key = GenericArray::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    let nonce = GenericArray::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    Ok(String::from_utf8(plaintext)?)
}
 */