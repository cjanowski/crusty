use crate::models::{Media, CreateMediaDto};
use crate::utils::EncryptionService;
use sqlx::{MySqlPool, Row};
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use log::{debug, error};

pub struct MediaService {
    pool: MySqlPool,
    encryption_service: EncryptionService,
    storage_path: PathBuf,
}

impl MediaService {
    pub fn new(pool: MySqlPool, storage_path: PathBuf) -> Self {
        Self {
            pool,
            encryption_service: EncryptionService::new(),
            storage_path,
        }
    }

    pub async fn upload_media(
        &self,
        user_id: Uuid,
        media_dto: CreateMediaDto,
        file_data: Vec<u8>,
    ) -> Result<Media, Box<dyn Error>> {
        debug!("Starting media upload process for user: {}", user_id);
        
        // Generate a unique ID for the media
        let media_id = Uuid::new_v4();
        debug!("Generated media ID: {}", media_id);
        
        // Generate encryption key for this file
        debug!("Generating encryption key");
        let file_key = self.encryption_service.generate_key()?;
        
        // Encrypt the file
        debug!("Encrypting file data");
        let (encrypted_data, encryption_iv) = self.encryption_service.encrypt(&file_data, &file_key)?;
        debug!("File encrypted successfully. Size: {} bytes", encrypted_data.len());
        
        // Create directory if it doesn't exist
        let user_dir = self.storage_path.join(user_id.to_string());
        debug!("Creating user directory: {:?}", user_dir);
        fs::create_dir_all(&user_dir).await?;
        
        // Save encrypted file
        let file_path = user_dir.join(media_id.to_string());
        debug!("Saving encrypted file to: {:?}", file_path);
        fs::write(&file_path, &encrypted_data).await?;

        debug!("Storing media information in database");
        // Store media information in database
        match sqlx::query!(
            r#"INSERT INTO media (
                id, user_id, filename, file_size, mime_type, encrypted_key, encryption_iv
            ) VALUES (
                UUID_TO_BIN(?), UUID_TO_BIN(?), ?, ?, ?, ?, ?
            )"#,
            media_id.to_string(),
            user_id.to_string(),
            media_dto.filename,
            encrypted_data.len() as i64,
            media_dto.mime_type,
            file_key,
            encryption_iv.to_vec()
        )
        .execute(&self.pool)
        .await {
            Ok(_) => debug!("Media record inserted successfully"),
            Err(e) => {
                error!("Failed to insert media record: {}", e);
                return Err(e.into());
            }
        }

        debug!("Fetching inserted media record");
        // Fetch the inserted media
        let row = sqlx::query(
            r#"SELECT
                BIN_TO_UUID(id) as id,
                BIN_TO_UUID(user_id) as user_id,
                filename,
                file_size,
                mime_type,
                encrypted_key,
                encryption_iv,
                created_at,
                updated_at
            FROM media WHERE id = UUID_TO_BIN(?)"#
        )
        .bind(media_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        debug!("Creating Media object from database row");
        let media = Media::from_row(
            Uuid::parse_str(row.get("id")).unwrap(),
            Uuid::parse_str(row.get("user_id")).unwrap(),
            row.get("filename"),
            row.get("file_size"),
            row.get("mime_type"),
            row.get("encrypted_key"),
            row.get("encryption_iv"),
            row.get("created_at"),
            row.get("updated_at")
        );

        debug!("Media upload process completed successfully");
        Ok(media)
    }

    pub async fn get_media(&self, user_id: Uuid, media_id: Uuid) -> Result<(Media, Vec<u8>), Box<dyn Error>> {
        // Fetch media metadata
        let row = sqlx::query(
            r#"SELECT
                BIN_TO_UUID(id) as id,
                BIN_TO_UUID(user_id) as user_id,
                filename,
                file_size,
                mime_type,
                encrypted_key,
                encryption_iv,
                created_at,
                updated_at
            FROM media WHERE id = UUID_TO_BIN(?)"#
        )
        .bind(media_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or("Media not found")?;

        let media = Media::from_row(
            Uuid::parse_str(row.get("id")).unwrap(),
            Uuid::parse_str(row.get("user_id")).unwrap(),
            row.get("filename"),
            row.get("file_size"),
            row.get("mime_type"),
            row.get("encrypted_key"),
            row.get("encryption_iv"),
            row.get("created_at"),
            row.get("updated_at")
        );

        // Read encrypted file
        let file_path = self.storage_path
            .join(user_id.to_string())
            .join(media_id.to_string());
        let encrypted_data = fs::read(&file_path).await?;

        // Decrypt file
        let decrypted_data = self.encryption_service.decrypt(
            &encrypted_data,
            &media.encrypted_key,
            media.encryption_iv.as_slice().try_into()?
        )?;

        Ok((media, decrypted_data))
    }

    pub async fn list_user_media(&self, user_id: Uuid) -> Result<Vec<Media>, Box<dyn Error>> {
        let rows = sqlx::query(
            r#"SELECT
                BIN_TO_UUID(id) as id,
                BIN_TO_UUID(user_id) as user_id,
                filename,
                file_size,
                mime_type,
                encrypted_key,
                encryption_iv,
                created_at,
                updated_at
            FROM media
            WHERE user_id = UUID_TO_BIN(?)"#
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let media_list = rows.into_iter().map(|row| Media::from_row(
            Uuid::parse_str(row.get("id")).unwrap(),
            Uuid::parse_str(row.get("user_id")).unwrap(),
            row.get("filename"),
            row.get("file_size"),
            row.get("mime_type"),
            row.get("encrypted_key"),
            row.get("encryption_iv"),
            row.get("created_at"),
            row.get("updated_at")
        )).collect();

        Ok(media_list)
    }

    pub async fn delete_media(&self, user_id: Uuid, media_id: Uuid) -> Result<(), Box<dyn Error>> {
        // Delete from database first
        let result = sqlx::query!(
            "DELETE FROM media WHERE id = UUID_TO_BIN(?) AND user_id = UUID_TO_BIN(?)",
            media_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err("Media not found".into());
        }

        // Delete file
        let file_path = self.storage_path
            .join(user_id.to_string())
            .join(media_id.to_string());
        
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }

        Ok(())
    }
} 