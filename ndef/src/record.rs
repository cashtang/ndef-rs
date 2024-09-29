use crate::*;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::borrow::Cow;
use std::io::{prelude::*, Cursor};

#[derive(Debug, Clone)]
pub struct NdefRecord {
    flags: RecordFlags,
    /// The Type Name Format (TNF) field of the record.
    tnf: TNF,
    /// The type field of the record.
    record_type: Vec<u8>,
    /// The ID field of the record.
    id: Option<Vec<u8>>,
    /// The payload field of the record.
    payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct RecordUri {
    abbrev: UriAbbrev,
    uri: Cow<'static, str>,
}

impl RecordUri {
    pub fn static_with_abbrev(abbrev: UriAbbrev, uri: &'static str) -> Self {
        Self {
            abbrev,
            uri: Cow::Borrowed(uri),
        }
    }

    pub fn from_static(uri: &'static str) -> Self {
        let (abbrev, uri) = Self::guess_abbrev(uri);
        Self {
            abbrev,
            uri: Cow::Borrowed(uri),
        }
    }

    pub fn with_abbrev<T: Into<String>>(abbrev: UriAbbrev, uri: T) -> Self {
        Self {
            abbrev,
            uri: Cow::Owned(uri.into()),
        }
    }

    pub fn from_string<T: Into<String>>(uri: T) -> Self {
        let uri = uri.into();
        let (abbrev, uri) = Self::guess_abbrev(&uri);
        Self {
            abbrev,
            uri: Cow::Owned(uri.to_owned()),
        }
    }

    fn guess_abbrev<'a>(uri: &'a str) -> (UriAbbrev, &'a str) {
        for abbr in URI_ABBREVIATIONS.iter() {
            if abbr == &NONE_ABBRE {
                continue;
            }
            if uri.starts_with(abbr.1) {
                return (*abbr, &uri[abbr.1.len()..]);
            }
        }
        (NONE_ABBRE, uri)
    }

    pub fn abbreviation(&self) -> UriAbbrev {
        self.abbrev.clone()
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
    
    pub fn full_uri(&self) -> String {
        if self.abbrev == NONE_ABBRE {
            return self.uri.to_string();
        }
        format!("{}{}", self.abbrev.as_uri(), self.uri)
    }
}

impl From<RecordUri> for Vec<u8> {
    fn from(record: RecordUri) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.push(record.abbrev.as_byte());
        buffer.extend_from_slice(record.uri.as_bytes());
        buffer
    }
}

#[allow(dead_code)]
impl NdefRecord {
    pub fn builder() -> NdefRecordBuilder {
        NdefRecordBuilder::new()
    }
    pub fn flags(&self) -> RecordFlags {
        self.flags
    }

    pub fn tnf(&self) -> TNF {
        self.tnf
    }

    pub fn record_type(&self) -> &[u8] {
        &self.record_type
    }

    pub fn id(&self) -> Option<&[u8]> {
        self.id.as_deref()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn uri_payload(&self) -> Option<RecordUri> {
        if self.payload.is_empty() {
            return None;
        }
        if RTD_URI == self.record_type {
            let abbrev = get_uri_abbreviation(self.payload[0]).map(|a| a.clone());
            if abbrev.is_none() {
                return None;
            }
            let abbrev = abbrev.unwrap();
            let uri = std::str::from_utf8(&self.payload[1..]).ok()?;
            Some(RecordUri::with_abbrev(abbrev, uri))
        } else {
            None
        }
    }

    pub fn clear_begin(&mut self) {
        self.flags.remove(RecordFlags::MB);
    }

    pub fn clear_start(&mut self) {
        self.flags.remove(RecordFlags::ME);
    }

    pub fn to_buffer(&self, flag: RecordFlags) -> Result<Vec<u8>> {
        let buffer: Vec<u8> = vec![];
        let mut output = Cursor::new(buffer);
        let mut rf = self.flags;
        if flag & RecordFlags::MB == RecordFlags::MB {
            rf |= RecordFlags::MB;
        } else {
            rf &= !RecordFlags::MB;
        }

        if flag & RecordFlags::ME == RecordFlags::ME {
            rf |= RecordFlags::ME;
        } else {
            rf &= !RecordFlags::ME;
        }

        let flag = rf.bits() | ((self.tnf as u8) & 0x07);

        output
            .write_u8(flag)
            .map_err(|_| anyhow!("Failed to write flags"))?;

        output
            .write_u8(self.record_type.len() as u8)
            .map_err(|_| anyhow!("Failed to write record type length"))?;
        if self.flags & RecordFlags::SR == RecordFlags::SR {
            output
                .write_u8(self.payload.len() as u8)
                .map_err(|_| anyhow!("Failed to write ID length"))?;
        } else {
            output
                .write_u32::<LittleEndian>(self.payload.len() as u32)
                .map_err(|_| anyhow!("Failed to write payload length"))?;
        }
        if let Some(id) = self.id.as_ref() {
            output
                .write_u8((id.len() & 0xff) as u8)
                .map_err(|_| anyhow!("Failed to write TNF"))?;
        }
        output
            .write_all(&self.record_type)
            .map_err(|_| anyhow!("Failed to write record type"))?;
        if let Some(id) = self.id.as_ref() {
            output
                .write_all(id)
                .map_err(|_| anyhow!("Failed to write ID"))?;
        }
        output
            .write_all(&self.payload)
            .map_err(|_| anyhow!("Failed to write payload"))?;
        Ok(output.into_inner())
    }

    pub(crate) fn decode(reader: &mut dyn Read) -> Result<Self> {
        let flags = reader.read_u8().map_err(|e| anyhow!("read error, {}", e))?;
        let tnf = TNF::from_repr(flags & 0x0f)
            .ok_or_else(|| anyhow!("unknown tnf <{:02x}>", (flags & 0x0f)))?;
        let flags = RecordFlags::from_bits_retain(flags);

        let type_len = reader
            .read_u8()
            .map_err(|_| anyhow!("read type length error"))?;
        let payload_len = if flags & RecordFlags::SR == RecordFlags::SR {
            reader
                .read_u8()
                .map_err(|_| anyhow!("read payload length error"))? as u32
        } else {
            reader
                .read_u32::<LittleEndian>()
                .map_err(|_| anyhow!("read payload length error"))?
        };

        let id_len = if flags & RecordFlags::IL == RecordFlags::IL {
            reader
                .read_u8()
                .map_err(|_| anyhow!("read id length error"))?
        } else {
            0
        };

        let mut record_type = vec![0u8; type_len as usize];
        reader
            .read_exact(&mut record_type)
            .map_err(|_| anyhow!("read record type error"))?;

        let id = if id_len > 0 {
            let mut id = vec![0u8; id_len as usize];
            reader
                .read_exact(&mut id)
                .map_err(|_| anyhow!("read record id error"))?;
            Some(id)
        } else {
            None
        };

        let mut payload = vec![0u8; payload_len as usize];
        reader
            .read_exact(&mut payload)
            .map_err(|_| anyhow!("read payload error"))?;
        Ok(Self {
            flags,
            tnf,
            id,
            record_type,
            payload,
        })
    }
}

pub struct NdefRecordBuilder {
    flags: RecordFlags,
    tnf: TNF,
    record_type: Vec<u8>,
    id: Option<Vec<u8>>,
    payload: Vec<u8>,
}

impl NdefRecordBuilder {
    fn new() -> Self {
        Self {
            flags: RecordFlags::empty(),
            tnf: TNF::Empty,
            record_type: vec![],
            id: None,
            payload: vec![],
        }
    }

    pub fn id(mut self, id: Vec<u8>) -> Self {
        if id.len() == 0 {
            return self;
        }
        self.id = Some(id);
        self.flags |= RecordFlags::IL;
        self
    }

    pub fn tnf(mut self, tnf: TNF) -> Self {
        self.tnf = tnf;
        self
    }

    pub fn payload<T, U>(mut self, record_type: T, payload: U) -> Self
    where
        T: Into<Vec<u8>>,
        U: Into<Vec<u8>>,
    {
        self.payload = payload.into();
        self.record_type = record_type.into();
        if self.payload.len() < 256 {
            self.flags |= RecordFlags::SR;
        }
        self
    }

    pub fn uri_payload(self, uri: RecordUri) -> Self {
        self.payload(RTD_URI, uri)
    }

    pub fn build(self) -> Result<NdefRecord> {
        if self.tnf == TNF::Empty
            && (!self.payload.is_empty() || !self.record_type.is_empty() || self.id.is_some())
        {
            return Err(anyhow!("Invalid empty record"));
        }
        if self.tnf == TNF::Empty {
            Ok(NdefRecord {
                flags: self.flags,
                tnf: self.tnf,
                record_type: vec![],
                id: None,
                payload: vec![],
            })
        } else {
            Ok(NdefRecord {
                flags: self.flags,
                tnf: self.tnf,
                record_type: self.record_type,
                id: self.id,
                payload: self.payload,
            })
        }
    }
}
