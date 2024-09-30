
pub mod record;
pub mod payload;
pub mod message;
pub mod tag;
pub mod error;
mod consts;


pub use consts::*;

pub type Result<T> = std::result::Result<T, error::NdefError>;

pub use record::NdefRecord;
pub use message::NdefMessage;