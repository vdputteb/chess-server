use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/**
 * Represents a user in the database with authentication and profile data
 * 
 * @field id         Unique identifier for the user
 * @field username   Unique username for login and display
 * @field email      Unique email address for communication
 * @field password   Hashed password for authentication
 * @field created_at Timestamp of account creation in UTC
 * @field pp_link    Profile picture link with default value
 * @author           Vander Putten Benjamin
 */
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub pp_link: String,
}

/**
 * Represents a chess game between two players with game data
 * 
 * @field id              Unique identifier for the game
 * @field white_player_id Foreign key referencing the white player's user ID (nullable)
 * @field black_player_id Foreign key referencing the black player's user ID (nullable)
 * @field pgn             Portable Game Notation string containing move history
 * @field result          Game outcome (e.g., "1-0", "0-1", "1/2-1/2")
 * @field created_at      Timestamp of game creation in UTC
 * @author               Vander Putten Benjamin
 */
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Game {
    pub id: i64,
    pub white_player_id: Option<i64>,
    pub black_player_id: Option<i64>,
    pub pgn: Option<String>,
    pub result: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/**
 * Database connection handler providing CRUD operations for chess application
 * 
 * @field pool SQLite connection pool for database operations
 * @author     Vander Putten Benjamin
 */
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /**
     * Creates a new database connection and initializes required tables
     * 
     * @param database_url SQLite database connection string
     * @return            Database instance or error if connection fails
     * @author            Vander Putten Benjamin
     */
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .connect(database_url)
            .await?;

        Self::create_tables(&pool).await?;

        Ok(Database { pool })
    }

    // CREATE

    /**
     * Creates database tables for users and games if they don't exist
     * 
     * @param pool SQLite connection pool reference
     * @return     Result indicating success or failure
     * @author     Vander Putten Benjamin
     */
    async fn create_tables(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                connected TINYINT(1) DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                pp_link TEXT DEFAULT './static/img/people.jpg'
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                white_player_id INTEGER,
                black_player_id INTEGER,
                pgn TEXT,
                result TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (white_player_id) REFERENCES users (id),
                FOREIGN KEY (black_player_id) REFERENCES users (id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        println!("Tables créées ou déjà existantes");
        Ok(())
    }

    /**
     * Creates a new user in the database with provided credentials
     * 
     * @param username      Unique username for the new user
     * @param email         Unique email address for the new user
     * @param password_hash Hashed password for authentication
     * @return              Created User object or error if creation fails
     * @author              Vander Putten Benjamin
     */
    pub async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password) VALUES (?, ?, ?) RETURNING *",
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /**
     * Creates a new game between two players in the database
     * 
     * @param   white_player_id User ID of the player with white pieces
     * @param   black_player_id User ID of the player with black pieces
     * @return  Created Game object or error if creation fails
     * @author  Vander Putten Benjamin
     */
    pub async fn create_game(&self, white_player_id: i64, black_player_id: i64) -> Result<Game> {
        let game = sqlx::query_as::<_, Game>(
            "INSERT INTO games (white_player_id, black_player_id) VALUES (?, ?) RETURNING *",
        )
        .bind(white_player_id)
        .bind(black_player_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(game)
    }

    // READ

    /**
     * Retrieves a user from the database by ID and username
     * 
     * @param id       User ID to search for
     * @param username Username to search for
     * @return         User object or error if not found
     * @author         Vander Putten Benjamin
     */
    pub async fn get_user(&self, id: i64, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND username = ?")
            .bind(id)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /**
     * Retrieves a user from the database by username only
     * 
     * @param username Username to search for
     * @return         User object or error if not found
     * @author         Vander Putten Benjamin
     */
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /**
     * Retrieves a user from the database by email address
     * 
     * @param email Email address to search for
     * @return      Optional User object if found, error if query fails
     * @author      Vander Putten Benjamin
     */
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }


    /**
     * Retrieves complete user information from the database by user ID
     * 
     * @param id User ID to search for
     * @return   User object containing id, username, email and creation date
     * @author   Vander Putten Benjamin
     */
    pub async fn get_user_info(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT id, username, email, password, created_at, pp_link FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /**
     * Retrieves the profile picture link for a user by their ID
     * 
     * @param id User ID to search for
     * @return   User object containing the profile picture link or error if not found
     * @author   Vander Putten Benjamin
     */
    pub async fn get_user_pp(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT id, username, email, password, created_at, pp_link FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    /**
     * Retrieves the hashed password for a user by username
     * 
     * @param username Username to search for
     * @return         User object containing password hash or error if not found
     * @author         Vander Putten Benjamin
     */
    pub async fn get_password(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT id, username, email, password, created_at, pp_link FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }
    /* 
    pub async fn is_user_connected(&self, id : i64) -> Result<bool> {
        let user = sqlx::query_as<_, bool> ("SELECT connected FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        return user;
    }
    */

    /**
     * Retrieves all games for a user by their ID (both as white and black player)
     * 
     * @param id User ID to search games for
     * @return   Vector of Game objects or error if query fails
     * @author   Vander Putten Benjamin
     */
    pub async fn get_user_games(&self, id: i64) -> Result<Vec<Game>> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT * FROM games WHERE white_player_id = ? OR black_player_id = ? ORDER BY created_at DESC",
        )
        .bind(id)
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(games)
    }

    // UPDATE

    /**
     * Updates user information including username, email and password with verification
     * 
     * @param id           User ID to update
     * @param username     New username value
     * @param email        New email address
     * @param old_password Current password for verification
     * @param new_password New password to set
     * @return             Updated User object or error if update fails
     * @author             Vander Putten Benjamin
     */
    pub async fn update_user(
        &self,
        id: i64,
        username: &str,
        email: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET username = ?, email = ?, password = ? WHERE id = ? AND password = ? RETURNING *",
        )
        .bind(username)
        .bind(email)
        .bind(new_password)
        .bind(id)
        .bind(old_password)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /**
     * Updates game information with PGN notation and result
     * 
     * @param game_id Game ID to update
     * @param pgn     Portable Game Notation string containing move history
     * @param result  Game outcome (e.g., "1-0", "0-1", "1/2-1/2")
     * @return        Updated Game object or error if update fails
     * @author        Vander Putten Benjamin
     */
    pub async fn update_game(&self, game_id: i64, pgn: &str, result: &str) -> Result<Option<Game>> {
        let game = sqlx::query_as::<_, Game>(
            "UPDATE games SET pgn = ?, result = ? WHERE id = ? RETURNING *",
        )
        .bind(pgn)
        .bind(result)
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(game)
    }

    /**
     * Updates user profile picture link
     * 
     * @param id   User ID to update
     * @param link New profile picture link
     * @return     Updated User object or error if update fails
     * @author     Vander Putten Benjamin
     */
    pub async fn update_pp(&self, id: i64, link: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("UPDATE users SET pp_link = ? WHERE id = ? RETURNING *")
            .bind(link)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    // DELETE

    /**
     * Deletes a user account after verifying credentials and updates game references to NULL
     * 
     * @param id       User ID to delete
     * @param username Username for verification
     * @param password Password for verification
     * @return         Deleted User object or error if deletion fails
     * @author         Vander Putten Benjamin
     */
    pub async fn delete_user(&self, id: i64, username: &str, password: &str) -> Result<Option<User>> {
        self.update_games_for_deleted_user(id).await?;
        
        let user = sqlx::query_as::<_, User>(
            "DELETE FROM users WHERE id = ? AND username = ? AND password = ? RETURNING *",
        )
        .bind(id)
        .bind(username)
        .bind(password)
        .fetch_optional(&self.pool)
        .await?;
        if user.is_some() {
            self.delete_orphaned_games().await?;
        }


        Ok(user)
    }

    /**
     * Updates game references to NULL when a user is deleted
     * 
     * @param user_id User ID that is being deleted
     * @return        Result indicating success or failure
     * @author        Vander Putten Benjamin
     */
    async fn update_games_for_deleted_user(&self, user_id: i64) -> Result<()> {
        // Mettre à jour les parties où l'utilisateur était le joueur blanc
        sqlx::query(
            "UPDATE games SET white_player_id = NULL WHERE white_player_id = ?",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Mettre à jour les parties où l'utilisateur était le joueur noir
        sqlx::query(
            "UPDATE games SET black_player_id = NULL WHERE black_player_id = ?",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /**
     * Deletes a specific game from the database by its ID
     * 
     * @param game_id Game ID to delete
     * @return        Deleted Game object or error if deletion fails
     * @author        Vander Putten Benjamin
     */
    pub async fn delete_game(&self, game_id: i64) -> Result<Option<Game>> {
        let game = sqlx::query_as::<_, Game>("DELETE FROM games WHERE id = ? RETURNING *")
            .bind(game_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(game)
    }

    /**
     * Cleans up orphaned games that have no valid player references
     * 
     * @return Result indicating success or failure of cleanup operation
     * @author Vander Putten Benjamin
     */
    async fn delete_orphaned_games(&self) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM games 
            WHERE white_player_id NOT IN (SELECT id FROM users)
            AND black_player_id NOT IN (SELECT id FROM users)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

}