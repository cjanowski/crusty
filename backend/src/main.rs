use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::path::PathBuf;
use std::net::TcpListener;

mod models;
mod routes;
mod services;
mod utils;

use services::auth_service::AuthService;
use services::media_service::MediaService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init();

    // Get environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let storage_path = env::var("STORAGE_PATH")
        .unwrap_or_else(|_| "storage".to_string());

    println!("Initializing server...");
    println!("Storage path: {}", storage_path);

    // Set up database connection pool
    println!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    println!("Database connected successfully");

    // Create services
    let auth_service = web::Data::new(AuthService::new(pool.clone(), jwt_secret));
    let media_service = web::Data::new(MediaService::new(
        pool.clone(),
        PathBuf::from(&storage_path),
    ));

    // Ensure storage directory exists
    std::fs::create_dir_all(&storage_path)?;
    println!("Storage directory created/verified");

    // Try to bind to the port first
    let bind_addr = "127.0.0.1:5000";
    println!("Attempting to bind to {}", bind_addr);
    
    let listener = match TcpListener::bind(bind_addr) {
        Ok(l) => {
            println!("Successfully bound to {}", bind_addr);
            l
        },
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", bind_addr, e);
            std::process::exit(1);
        }
    };

    println!("Starting server on {}", bind_addr);

    HttpServer::new(move || {
        println!("Configuring server instance...");
        App::new()
            .app_data(auth_service.clone())
            .app_data(media_service.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Frame-Options", "DENY"))
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("X-XSS-Protection", "1; mode=block")))
            .wrap(middleware::Compress::default())
            .configure(routes::auth::config)
            .configure(routes::media::config)
            .route("/health", web::get().to(|| async { "Server is running!" }))
    })
    .workers(2)
    .listen(listener)?
    .run()
    .await
} 