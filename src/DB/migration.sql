-- migrations.sql
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    profile_image_path TEXT DEFAULT '/static/img/people.jpg'
);

CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    white_player_id INTEGER NOT NULL,
    black_player_id INTEGER NOT NULL,
    pgn TEXT,
    result TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (white_player_id) REFERENCES users (id),
    FOREIGN KEY (black_player_id) REFERENCES users (id)
);

