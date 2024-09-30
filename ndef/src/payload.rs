use crate::*;
use std::borrow::Cow;
use std::convert::From;
#[cfg(feature = "mime")]
use mime::Mime;
use crate::{error::NdefError, record::NdefRecord};

pub trait RecordPayload {
    fn record_type(&self) -> Cow<'_, [u8]>;
    fn payload(&self) -> Cow<'_, [u8]>;
}

#[derive(Debug, PartialEq)]
pub struct UriPayload {
    abbrev: UriAbbrev,
    uri: Cow<'static, str>,
}

impl UriPayload {
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

    fn guess_abbrev(uri: &str) -> (UriAbbrev, &str) {
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

impl TryFrom<&NdefRecord> for UriPayload {
    type Error = crate::error::NdefError;

    fn try_from(record: &NdefRecord) -> Result<Self> {
        if record.tnf() != TNF::WellKnown {
            return Err(NdefError::InvalidTnf);
        }
        if record.record_type() != RTD_URI.as_bytes() {
            return Err(NdefError::InvalidRecordType);
        }
        let payload = record.payload();
        let abbrev = get_uri_abbreviation(payload[0]).unwrap_or_else(|| &NONE_ABBRE);
        let uri = std::str::from_utf8(&payload[1..]).map_err(|_| NdefError::InvalidEncoding)?;
        Ok(UriPayload {
            abbrev: abbrev.clone(),
            uri: Cow::Owned(uri.to_string()),
        })
    }
}

impl From<&UriPayload> for Vec<u8> {
    fn from(record: &UriPayload) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.push(record.abbrev.as_byte());
        buffer.extend_from_slice(record.uri.as_bytes());
        buffer
    }
}

impl RecordPayload for UriPayload {
    fn record_type(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(RTD_URI.as_bytes())
    }

    fn payload(&self) -> Cow<'_, [u8]> {
        let r = self.into();
        Cow::Owned(r)
    }
}

#[derive(Debug, PartialEq)]
pub struct TextPayload {
    text: Cow<'static, str>,
}

impl TextPayload {
    pub fn from_static(text: &'static str) -> Self {
        Self {
            text: Cow::Borrowed(text),
        }
    }

    pub fn from_string<T: Into<String>>(text: T) -> Self {
        Self {
            text: Cow::Owned(text.into()),
        }
    }
}

impl RecordPayload for TextPayload {
    fn record_type(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(RTD_TEXT.as_bytes())
    }

    fn payload(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.text.as_bytes())
    }
}

impl TryFrom<&NdefRecord> for TextPayload {
    type Error = crate::error::NdefError;

    fn try_from(record: &NdefRecord) -> Result<Self> {
        if record.tnf() != TNF::WellKnown {
            return Err(NdefError::InvalidTnf);
        }
        if record.record_type() != RTD_TEXT.as_bytes() {
            return Err(NdefError::InvalidRecordType);
        }
        let payload = record.payload();
        let text = std::str::from_utf8(&payload).map_err(|_| NdefError::InvalidEncoding)?;
        Ok(TextPayload {
            text: Cow::Owned(text.to_string()),
        })
    }
}

pub struct SmartPosterPayload {
    data: Cow<'static, [u8]>,
}

impl SmartPosterPayload {
    pub fn from_static(data: &'static [u8]) -> Self {
        Self {
            data: Cow::Borrowed(data),
        }
    }

    pub fn from_string<T: Into<Vec<u8>>>(data: T) -> Self {
        Self {
            data: Cow::Owned(data.into()),
        }
    }
}

impl RecordPayload for SmartPosterPayload {
    fn record_type(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(RTD_SMART_POSTER.as_bytes())
    }

    fn payload(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.data)
    }
}

impl TryFrom<&NdefRecord> for SmartPosterPayload {
    type Error = crate::error::NdefError;

    fn try_from(record: &NdefRecord) -> Result<Self> {
        if record.tnf() != TNF::WellKnown {
            return Err(NdefError::InvalidTnf);
        }
        if record.record_type() != RTD_SMART_POSTER.as_bytes() {
            return Err(NdefError::InvalidRecordType);
        }
        Ok(SmartPosterPayload {
            data: Cow::Owned(record.payload().to_vec()),
        })
    }
}

pub struct ExternalPayload {
    record_type: Cow<'static, [u8]>,
    payload: Cow<'static, [u8]>,
}

impl ExternalPayload {
    pub fn from_static(rt: &'static [u8], pl: &'static [u8]) -> Self {
        Self {
            record_type: Cow::Borrowed(rt),
            payload: Cow::Borrowed(pl),
        }
    }

    pub fn from_raw<T, U>(record_type: T, payload: U) -> Self
    where
        T: Into<Vec<u8>>,
        U: Into<Vec<u8>>,
    {
        Self {
            record_type: Cow::Owned(record_type.into()),
            payload: Cow::Owned(payload.into()),
        }
    }
}

impl RecordPayload for ExternalPayload {
    fn record_type(&self) -> Cow<'_, [u8]> {
        self.record_type.clone()
    }

    fn payload(&self) -> Cow<'_, [u8]> {
        self.payload.clone()
    }
}

impl TryFrom<&NdefRecord> for ExternalPayload {
    type Error = crate::error::NdefError;

    fn try_from(record: &NdefRecord) -> Result<Self> {
        if record.tnf() != TNF::External {
            return Err(NdefError::InvalidTnf);
        }
        Ok(ExternalPayload {
            record_type: Cow::Owned(record.record_type().to_vec()),
            payload: Cow::Owned(record.payload().to_vec()),
        })
    }
}


#[cfg(feature = "mime")]
pub struct MimePayload {
    mime_type: Mime,
    payload: Cow<'static, [u8]>,
}

#[cfg(feature = "mime")]
impl MimePayload {
    pub fn from_mime<U>(mime: Mime, payload: U) -> Self
    where
        U: Into<Vec<u8>>,
    {
        Self {
            mime_type: mime,
            payload: Cow::Owned(payload.into()),
        }
    }
}

#[cfg(feature = "mime")]
impl RecordPayload for MimePayload {
    fn record_type(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.mime_type.type_().as_ref().as_bytes().to_vec())
    }

    fn payload(&self) -> Cow<'_, [u8]> {
        self.payload.clone()
    }
}

#[cfg(feature = "mime")]
impl TryFrom<&NdefRecord> for MimePayload {
    type Error = crate::error::NdefError;

    fn try_from(record: &NdefRecord) -> Result<Self> {
        if record.tnf() != TNF::MimeMedia {
            return Err(NdefError::InvalidTnf);
        }
        let mime_type = record.record_type();
        let mime_type = std::str::from_utf8(&mime_type).map_err(|_| NdefError::InvalidEncoding)?;
        let mime_type = mime_type.parse().map_err(|_| NdefError::InvalidMime)?;
        Ok(MimePayload {
            mime_type,
            payload: Cow::Owned(record.payload().to_vec()),
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_uri() {
        let uri = UriPayload::from_static("https://www.sina.com.cn");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!(uri.uri(), "sina.com.cn");
        assert_eq!(uri.full_uri(), "https://www.sina.com.cn");
        assert_eq!(RTD_URI.as_bytes(), uri.record_type().as_ref());
        assert_eq!(b"\x02sina.com.cn".to_vec().as_slice(), uri.payload().as_ref());

        let uri = UriPayload::static_with_abbrev(HTTPS_WWW, "sina.com.cn");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!(uri.uri(), "sina.com.cn");

        let uri = UriPayload::static_with_abbrev(HTTP_WWW, "http://www.baidu.com");
        assert_eq!(HTTP_WWW, uri.abbreviation());
        assert_eq!("http://www.baidu.com", uri.uri());

        let uri = UriPayload::from_string("https://www.google.com");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!("google.com", uri.uri());

        let uri = UriPayload::from_static("weixin://dl/12321");
        assert_eq!(NONE_ABBRE, uri.abbreviation());
        assert_eq!("weixin://dl/12321", uri.uri());
    }

    #[test]
    fn test_text() {
        let text = TextPayload::from_static("Hello, World!");
        assert_eq!(RTD_TEXT.as_bytes(), text.record_type().as_ref());
        assert_eq!(b"Hello, World!", text.payload().as_ref());
    }

    #[test]
    fn test_smart_poster() {
        let sp = SmartPosterPayload::from_static(&[0x00, 0x01, 0x02, 0x03]);
        assert_eq!(RTD_SMART_POSTER.as_bytes(), sp.record_type().as_ref());
        assert_eq!(&[0x00, 0x01, 0x02, 0x03], sp.payload().as_ref());
    }

}