use crate::models::{User, CreateUserDto, LoginDto};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use std::error::Error;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{debug, error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // user id
    pub exp: usize,   // expiration time
    pub iat: usize,   // issued at
}

pub struct AuthService {
    pool: MySqlPool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: MySqlPool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn register(&self, user_dto: CreateUserDto) -> Result<User, Box<dyn Error>> {
        debug!("Starting registration for user: {}", user_dto.email);
        
        // Check if user exists
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE email = ? OR username = ?",
            user_dto.email,
            user_dto.username
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_some() {
            debug!("User already exists with email: {} or username: {}", user_dto.email, user_dto.username);
            return Err("User already exists".into());
        }

        debug!("Hashing password...");
        // Hash password
        let password_hash = match hash(user_dto.password.as_bytes(), DEFAULT_COST) {
            Ok(hash) => hash,
            Err(e) => {
                error!("Failed to hash password: {}", e);
                return Err("Failed to hash password".into());
            }
        };
        
        let id = Uuid::new_v4();
        debug!("Generated UUID: {}", id);

        // Insert new user
        debug!("Inserting new user into database...");
        match sqlx::query!(
            r#"INSERT INTO users (id, username, email, password_hash)
            VALUES (UUID_TO_BIN(?, true), ?, ?, ?)"#,
            id.to_string(),
            user_dto.username,
            user_dto.email,
            password_hash
        )
        .execute(&self.pool)
        .await {
            Ok(_) => debug!("Successfully inserted user"),
            Err(e) => {
                error!("Failed to insert user: {}", e);
                return Err("Failed to create user".into());
            }
        }

        // Fetch the inserted user
        debug!("Fetching newly created user...");
        let row = match sqlx::query(
            r#"SELECT 
                BIN_TO_UUID(id, true) as id,
                username,
                email,
                password_hash,
                created_at,
                updated_at
            FROM users WHERE id = UUID_TO_BIN(?, true)"#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await {
            Ok(row) => row,
            Err(e) => {
                error!("Failed to fetch created user: {}", e);
                return Err("Failed to fetch created user".into());
            }
        };

        debug!("Creating User struct from row...");
        let user = User::from_row(
            Uuid::parse_str(row.get("id")).unwrap(),
            row.get("username"),
            row.get("email"),
            row.get("password_hash"),
            row.get("created_at"),
            row.get("updated_at")
        );

        debug!("Registration completed successfully");
        Ok(user)
    }

    pub async fn login(&self, login_dto: LoginDto) -> Result<String, Box<dyn Error>> {
        // Find user
        let row = sqlx::query(
            r#"SELECT 
                BIN_TO_UUID(id) as id,
                username,
                email,
                password_hash,
                created_at,
                updated_at
            FROM users WHERE email = ?"#
        )
        .bind(&login_dto.email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Invalid credentials")?;

        let user = User::from_row(
            Uuid::parse_str(row.get("id")).unwrap(),
            row.get("username"),
            row.get("email"),
            row.get("password_hash"),
            row.get("created_at"),
            row.get("updated_at")
        );

        // Verify password
        if !verify(login_dto.password.as_bytes(), &user.password_hash)? {
            return Err("Invalid credentials".into());
        }

        // Generate JWT token
        self.generate_token(&user.id.to_string())
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String, Box<dyn Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 24 * 3600, // 24 hours from now
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes())
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default()
        )?;

        Ok(token_data.claims)
    }
} 