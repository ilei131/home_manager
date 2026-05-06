use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub admin_default_password: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-in-production".to_string()),
            admin_default_password: env::var("ADMIN_DEFAULT_PASSWORD")
                .unwrap_or_else(|_| "admin123".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}
