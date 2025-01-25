use actix_web::{web, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use uuid::Uuid;
use std::io::Write;

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
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
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
                    
                    while let Some(chunk) = field.next().await {
                        match chunk {
                            Ok(data) => {
                                if let Err(_) = file_data.write_all(&data) {
                                    return HttpResponse::InternalServerError().json(ErrorResponse {
                                        error: "Failed to process file".to_string()
                                    });
                                }
                            }
                            Err(_) => {
                                return HttpResponse::InternalServerError().json(ErrorResponse {
                                    error: "Failed to read file".to_string()
                                });
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    if file_data.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No file provided".to_string()
        });
    }

    let media_dto = CreateMediaDto {
        filename,
        mime_type: content_type,
    };

    match media_service.upload_media(user_id, media_dto, file_data).await {
        Ok(media) => HttpResponse::Created().json(MediaResponse {
            id: media.id,
            filename: media.filename,
            mime_type: media.mime_type,
            created_at: media.created_at,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to upload file: {}", e)
        }),
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
        Ok(media_list) => {
            let response: Vec<MediaResponse> = media_list
                .into_iter()
                .map(|m| MediaResponse {
                    id: m.id,
                    filename: m.filename,
                    mime_type: m.mime_type,
                    created_at: m.created_at,
                })
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to list media: {}", e)
        }),
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