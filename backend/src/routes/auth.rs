use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use crate::models::{CreateUserDto, LoginDto};
use crate::services::auth_service::AuthService;

#[derive(Serialize)]
struct AuthResponse {
    token: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}

async fn register(
    user_dto: web::Json<CreateUserDto>,
    auth_service: web::Data<AuthService>,
) -> impl Responder {
    match auth_service.register(user_dto.into_inner()).await {
        Ok(user) => {
            match auth_service.generate_token(&user.id.to_string()) {
                Ok(token) => HttpResponse::Created().json(AuthResponse { token }),
                Err(_) => HttpResponse::InternalServerError().json(ErrorResponse { 
                    error: "Failed to generate token".to_string() 
                })
            }
        },
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("already exists") {
                HttpResponse::Conflict().json(ErrorResponse { 
                    error: "User already exists".to_string() 
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse { 
                    error: "Failed to register user".to_string() 
                })
            }
        }
    }
}

async fn login(
    login_dto: web::Json<LoginDto>,
    auth_service: web::Data<AuthService>,
) -> impl Responder {
    match auth_service.login(login_dto.into_inner()).await {
        Ok(token) => HttpResponse::Ok().json(AuthResponse { token }),
        Err(_) => HttpResponse::Unauthorized().json(ErrorResponse { 
            error: "Invalid credentials".to_string() 
        })
    }
} 