use serde::{Deserialize, Serialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    #[serde(serialize_with = "serialize_uuid")]
    pub id: Uuid,
    #[serde(serialize_with = "serialize_uuid")]
    pub user_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub mime_type: String,
    #[serde(skip_serializing)]
    pub encrypted_key: Vec<u8>,
    #[serde(skip_serializing)]
    pub encryption_iv: Vec<u8>,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl Media {
    pub fn from_row(
        id: Uuid,
        user_id: Uuid,
        filename: String,
        file_size: i64,
        mime_type: String,
        encrypted_key: Vec<u8>,
        encryption_iv: Vec<u8>,
        created_at: Option<OffsetDateTime>,
        updated_at: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            id,
            user_id,
            filename,
            file_size,
            mime_type,
            encrypted_key,
            encryption_iv,
            created_at,
            updated_at,
        }
    }
}

fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateMediaDto {
    pub filename: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct MediaResponse {
    #[serde(serialize_with = "serialize_uuid")]
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub created_at: Option<OffsetDateTime>,
} 