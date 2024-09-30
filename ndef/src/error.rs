use thiserror::Error;

#[derive(Error, Debug)]
pub enum NdefError {
    #[error("Invalid TNF value")]
    InvalidTnf,
    #[error("Invalid record type")]
    InvalidRecordType,
    #[error("Invalid payload")]
    InvalidPayload,
    #[error("Invalid record ID")]
    InvalidId,
    #[error("Invalid URI")]
    InvalidUri,
    #[error("Invalid MIME type")]
    InvalidMime,
    #[error("Invalid language code")]
    InvalidLanguage,
    #[error("Invalid encoding")]
    InvalidEncoding,
    #[error("Invalid record flags")]
    InvalidFlags,
    #[error("Invalid record")]
    InvalidRecord,
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Invalid tag")]
    InvalidTag,
    #[error("Invalid tag type")]
    InvalidTagType,
    #[error("Invalid tag data")]
    InvalidTagData,
    #[error("Invalid tag length")]
    InvalidTagLength,
    #[error("Invalid tag version")]
    InvalidTagVersion,
    #[error("Invalid tag memory size")]
    InvalidTagMemorySize,
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}