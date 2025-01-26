use actix_web::{web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use uuid::Uuid;
use std::io::Write;
use log::{debug, error};

use crate::models::{CreateMediaDto, MediaResponse};
use crate::services::media_service::MediaService;
use crate::services::auth_service::Claims;
use crate::utils::Auth;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/media")
            .wrap(Auth::new())
            .route("", web::get().to(list_media))
            .route("", web::post().to(upload_media))
            .route("/{media_id}", web::get().to(get_media))
            .route("/{media_id}", web::delete().to(delete_media))
    );
}

async fn upload_media(
    mut payload: Multipart,
    media_service: web::Data<MediaService>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    debug!("Starting media upload. Claims: {:?}", claims);
    
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => {
            debug!("Parsed user ID: {}", id);
            id
        },
        Err(e) => {
            error!("Failed to parse user ID: {}", e);
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string()
            });
        }
    };

    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut content_type = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        
        if let Some(name) = content_disposition.get_name() {
            match name {
                "file" => {
                    filename = content_disposition
                        .get_filename()
                        .unwrap_or("unknown")
                        .to_string();
                    content_type = field
                        .content_type()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    
                    debug!("Processing file: {} ({})", filename, content_type);
                    
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(data) => {
                                if let Err(e) = file_data.write_all(&data) {
                                    error!("Failed to write file data: {}", e);
                                    return HttpResponse::InternalServerError().json(ErrorResponse {
                                        error: "Failed to process file".to_string()
                                    });
                                }
                            }
                            Err(e) => {
                                error!("Failed to read file chunk: {}", e);
                                return HttpResponse::InternalServerError().json(ErrorResponse {
                                    error: "Failed to read file".to_string()
                                });
                            }
                        }
                    }
                    debug!("File processing complete. Size: {} bytes", file_data.len());
                }
                _ => continue,
            }
        }
    }

    if file_data.is_empty() {
        error!("No file data received");
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No file provided".to_string()
        });
    }

    let media_dto = CreateMediaDto {
        filename,
        mime_type: content_type,
    };

    debug!("Attempting to upload media for user {}", user_id);
    match media_service.upload_media(user_id, media_dto, file_data).await {
        Ok(media) => {
            debug!("Media upload successful. ID: {}", media.id);
            HttpResponse::Created().json(MediaResponse {
                id: media.id,
                filename: media.filename,
                mime_type: media.mime_type,
                created_at: media.created_at,
            })
        },
        Err(e) => {
            error!("Failed to upload media: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to upload file: {}", e)
            })
        },
    }
}

async fn get_media(
    path: web::Path<Uuid>,
    media_service: web::Data<MediaService>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let media_id = path.into_inner();
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string()
            });
        }
    };

    match media_service.get_media(user_id, media_id).await {
        Ok((media, data)) => {
            HttpResponse::Ok()
                .content_type(media.mime_type)
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", media.filename)))
                .body(data)
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "Media not found".to_string()
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Failed to retrieve media: {}", e)
                })
            }
        }
    }
}

async fn list_media(
    media_service: web::Data<MediaService>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string()
            });
        }
    };

    match media_service.list_user_media(user_id).await {
        Ok(media_list) => HttpResponse::Ok().json(media_list),
        Err(e) => {
            error!("Failed to list media: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to list media: {}", e)
            })
        },
    }
}

async fn delete_media(
    path: web::Path<Uuid>,
    media_service: web::Data<MediaService>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let media_id = path.into_inner();
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid user ID".to_string()
            });
        }
    };

    match media_service.delete_media(user_id, media_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not found") {
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "Media not found".to_string()
                })
            } else {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Failed to delete media: {}", e)
                })
            }
        }
    }
} 