use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Cursor, Write};
use anyhow::Result;

use crate::message::NdefMessage;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TlvTag {
    NULL = 0x00,
    LockControl = 0x01,
    MemoryControl = 0x02,
    NDEFMessage = 0x03,
    Proprietary = 0xFD,
    Terminator = 0xFE,
}

#[derive(Debug, Clone)]
pub struct TlvValue {
    tag: TlvTag,
    value: Option<Vec<u8>>,
}

impl TlvValue {
    pub fn terminator() -> Self {
        Self {
            tag: TlvTag::Terminator,
            value: None,
        }
    }

    pub fn null() -> Self {
        Self {
            tag: TlvTag::NULL,
            value: None,
        }
    }

    pub fn lock_control(value: &[u8]) -> Self {
        Self {
            tag: TlvTag::LockControl,
            value: Some(value.to_vec()),
        }
    }

    pub fn memory_control(value: &[u8]) -> Self {
        Self {
            tag: TlvTag::MemoryControl,
            value: Some(value.to_vec()),
        }
    }

    pub fn message(value: &[u8]) -> Self {
        let value = if value.is_empty() {
            Some(vec![])
        } else {
            Some(value.to_vec())
        };

        Self {
            tag: TlvTag::NDEFMessage,
            value,
        }
    }

    pub fn ndef_message(message: &NdefMessage) -> Result<Self> {
        let value = message.to_buffer()?;
        Ok(Self {
            tag: TlvTag::NDEFMessage,
            value: Some(value),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let buffer = vec![self.tag as u8];
        let mut writer = Cursor::new(buffer);
        writer.write_u8(self.tag as u8).unwrap();
        if let Some(value) = &self.value {
            if value.len() == 0 {
                writer.write_u8(0x00).unwrap();
            } else if value.len() < 0xff {
                writer.write_u8(value.len() as u8).unwrap();
                writer.write_all(&value).unwrap();
            } else {
                writer.write_u8(0xff).unwrap();
                writer
                    .write_u16::<LittleEndian>(value.len() as u16)
                    .unwrap();
                writer.write_all(&value).unwrap();
            }
        }
        writer.into_inner()
    }
}

pub struct NFT2Tag {
    cc: [u8; 4],
    tlvs: Vec<TlvValue>,
}

impl NFT2Tag {
    pub fn builder() -> TagBuilder {
        TagBuilder::new()
    }

    pub fn capacity_in_bytes(&self) -> u16 {
        self.cc[2] as u16 * 8
    }

    pub fn capacity(&self) -> u8 {
        self.cc[2]
    }

    pub fn version(&self) -> u8 {
        self.cc[1]
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        if self.capacity_in_bytes() > 2048 {
            return Err(anyhow::anyhow!("Invalid memory size"));
        }

        let buffer = self
            .tlvs
            .iter()
            .map(|v| v.to_bytes())
            .flatten()
            .collect::<Vec<_>>();
        if self.capacity_in_bytes() < (buffer.len() as u16) {
            return Err(anyhow::anyhow!("Invalid memory size"));
        }
        let header = self.cc.to_vec();
        Ok([header, buffer].concat())
    }
}

pub struct TagBuilder {
    nfc_header: u8,
    nfc_version: u8,
    memory_size: u8,
    access: u8,
    tlvs: Vec<TlvValue>,
}

impl TagBuilder {
    fn new() -> Self {
        Self {
            nfc_header: 0xe1,
            nfc_version: 0x10,
            memory_size: 0x00,
            access: 0x0f,
            tlvs: vec![],
        }
    }

    pub fn size_in_bytes(mut self, num_of_bytes: u16) -> Self {
        let n = (num_of_bytes - 1) / 8;
        self.memory_size = n as u8 + 1;
        self
    }

    pub fn size_in_8bytes(mut self, num_of_8bytes: u8) -> Self {
        self.memory_size = num_of_8bytes;
        self
    }

    pub fn access(mut self, read: u8, write: u8) -> Self {
        self.access = (read << 4) | write;
        self
    }

    pub fn add_tlv(mut self, value: TlvValue) -> Self {
        self.tlvs.push(value);
        self
    }

    pub fn build(self) -> NFT2Tag {
        NFT2Tag {
            cc: [
                self.nfc_header,
                self.nfc_version,
                self.memory_size,
                self.access,
            ],
            tlvs: self.tlvs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tlv_value() {
        let tlv = TlvValue::lock_control(&[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(tlv.tag, TlvTag::LockControl);
        assert_eq!(tlv.value, Some(vec![0x00, 0x00, 0x00, 0x00]));

        let tlv = TlvValue::memory_control(&[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(tlv.tag, TlvTag::MemoryControl);
        assert_eq!(tlv.value, Some(vec![0x00, 0x00, 0x00, 0x00]));

        let tlv = TlvValue::message(&[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(tlv.tag, TlvTag::NDEFMessage);
        assert_eq!(tlv.value, Some(vec![0x00, 0x00, 0x00, 0x00]));

        let tlv = TlvValue::message(&[]);
        assert_eq!(tlv.tag, TlvTag::NDEFMessage);
        assert_eq!(tlv.value, Some(vec![]));

        let tlv = TlvValue::terminator();
        assert_eq!(tlv.tag, TlvTag::Terminator);
        assert_eq!(tlv.value, None);

        let bytes = tlv.to_bytes();
        assert_eq!(bytes, vec![0xfe]);
    }

    #[test]
    fn test_empty() {
        let tag1 = TlvValue::message(&[]);
        let tag2 = TlvValue::terminator();
        let t2tag = NFT2Tag::builder()
            .size_in_bytes(48)
            .add_tlv(tag1)
            .add_tlv(tag2)
            .build();
        let bytes = t2tag.to_bytes().unwrap();
        let expect = "e110060f0300fe";
        assert_eq!(hex::decode(expect).unwrap(), bytes);
    }

    #[test]
    fn test_ndef_message() {
        use super::*;
        use crate::record::{NdefRecord, RecordUri};
        use crate::*;

        let record1 = NdefRecord::builder()
            .tnf(TNF::WellKnown)
            .uri_payload(RecordUri::from_static("weixin://dl/business"))
            .build()
            .unwrap();

        let record2 = NdefRecord::builder()
            .tnf(TNF::External)
            .payload(b"android.com:pkg", b"com.tencent.mm")
            .build()
            .unwrap();

        let message = NdefMessage::from(&[record1, record2]);
        let tlv = TlvValue::ndef_message(&message).unwrap();
        let t2tag = NFT2Tag::builder()
            .size_in_bytes(256)
            .add_tlv(tlv)
            .add_tlv(TlvValue::terminator())
            .build();
        let bytes = t2tag.to_bytes().unwrap();
        let expect = "e110200f0339910115550077656978696e3a2f2f646c2f627573696e657373540f0e616e64726f69642e636f6d3a706b67636f6d2e74656e63656e742e6d6dfe";
        assert_eq!(expect, hex::encode(bytes));
    }
}
