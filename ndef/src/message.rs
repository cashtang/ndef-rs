use crate::{record::NdefRecord, *};
use anyhow::{bail, Result};
use std::io::Cursor;

pub struct NdefMessage {
    records: Vec<NdefRecord>,
}

impl From<NdefRecord> for NdefMessage {
    fn from(record: NdefRecord) -> Self {
        Self {
            records: vec![record],
        }
    }
}

impl<T> From<T> for NdefMessage
where
    T: AsRef<[NdefRecord]>,
{
    fn from(records: T) -> Self {
        Self {
            records: records.as_ref().to_vec(),
        }
    }
}

impl NdefMessage {
    pub fn new() -> Self {
        Self { records: vec![] }
    }

    pub fn add_record(&mut self, record: NdefRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[NdefRecord] {
        &self.records
    }

    pub fn to_buffer(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![];
        for (index, record) in self.records.iter().enumerate() {
            let flag = if self.records.len() == 1 {
                RecordFlags::ME | RecordFlags::MB
            } else if index == 0 && self.records.len() > 1 {
                RecordFlags::MB
            } else if index == self.records.len() - 1 {
                RecordFlags::ME
            } else {
                RecordFlags::empty()
            };
            buffer.extend_from_slice(&record.to_buffer(flag)?);
        }
        Ok(buffer)
    }

    pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Self> {
        let total = data.as_ref().len() as u64;
        let mut reader = Cursor::new(data.as_ref());
        let mut records = vec![];
        loop {
            let record = NdefRecord::decode(&mut reader)?;
            if record.flags() & RecordFlags::MB == RecordFlags::MB && records.len() > 0 {
                bail!("record MB flag is set , but not first record");
            }
            let flags = record.flags();
            records.push(record);
            if reader.position() >= total {
                if flags & RecordFlags::ME != RecordFlags::ME {
                    bail!("record ME flag is not set")
                } 
                break;
            }
        }
        Ok(Self { records })
    }
}

#[cfg(test)]
mod tests {

    use crate::message::NdefMessage;
    use crate::record::{NdefRecord, RecordUri};
    use crate::*;

    #[test]
    fn test_record_uri() {
        let uri = RecordUri::from_static("https://www.sina.com.cn");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!(uri.uri(), "sina.com.cn");

        let uri = RecordUri::static_with_abbrev(HTTPS_WWW, "sina.com.cn");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!(uri.uri(), "sina.com.cn");

        let uri = RecordUri::static_with_abbrev(HTTP_WWW, "http://www.baidu.com");
        assert_eq!(HTTP_WWW, uri.abbreviation());
        assert_eq!("http://www.baidu.com", uri.uri());

        let uri = RecordUri::from_string("https://www.google.com");
        assert_eq!(HTTPS_WWW, uri.abbreviation());
        assert_eq!("google.com", uri.uri());

        let uri = RecordUri::from_static("weixin://dl/12321");
        assert_eq!(NONE_ABBRE, uri.abbreviation());
        assert_eq!("weixin://dl/12321", uri.uri());
    }

    #[test]
    fn test_multiple_records() {
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
        assert_eq!(message.records().len(), 2);
        let buffer = message.to_buffer().unwrap();
        let expect = "910115550077656978696e3a2f2f646c2f627573696e657373540f0e616e64726f69642e636f6d3a706b67636f6d2e74656e63656e742e6d6d";
        assert_eq!(expect, hex::encode(buffer));

        let message = NdefMessage::decode(hex::decode(expect).unwrap()).unwrap();

        assert_eq!(2, message.records().len());

        let record = message.records().get(0).unwrap();
        assert_eq!(TNF::WellKnown, record.tnf());
        assert_eq!(RTD_URI.as_bytes(), record.record_type());
        let payload = record.uri_payload().unwrap();
        assert_eq!(NONE_ABBRE, payload.abbreviation());
        assert_eq!("weixin://dl/business", payload.uri());
        assert_eq!("weixin://dl/business", payload.full_uri());

        let record = message.records().get(1).unwrap();
        assert_eq!(TNF::External, record.tnf());
        assert_eq!(b"android.com:pkg", record.record_type());
        assert_eq!(b"com.tencent.mm", record.payload());

        assert_eq!(None, record.uri_payload());

    }

    #[test]
    fn test_single_record() {
        let record = NdefRecord::builder()
            .tnf(TNF::WellKnown)
            .uri_payload(RecordUri::with_abbrev(
                HTTP_WWW,
                "supwisdom.com".to_string(),
            ))
            .build()
            .unwrap();
        let message = NdefMessage::from(record);
        let expect = "d1010e5501737570776973646f6d2e636f6d";
        assert_eq!(expect, hex::encode(message.to_buffer().unwrap()));

        let message = NdefMessage::decode(hex::decode(expect).unwrap()).unwrap();
        assert_eq!(1, message.records().len());
        let record = message.records().get(0).unwrap();
        assert_eq!(TNF::WellKnown , record.tnf());
        assert_eq!(RTD_URI.as_bytes(), record.record_type());
        let payload = record.uri_payload().unwrap();
        assert_eq!(HTTP_WWW, payload.abbreviation());
        assert_eq!("supwisdom.com", payload.uri());
        assert_eq!("http://www.supwisdom.com", payload.full_uri());
    }

    #[test]
    fn test_not_sr() {
        let record = NdefRecord::builder()
            .tnf(TNF::External)
            .payload(RTD_TEXT, [0xab; 300])
            .build()
            .unwrap();
        let message = NdefMessage::from(record);
        let buffer = message.to_buffer().unwrap();
        let expect = "c4012c01000054abababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababab";
        assert_eq!(expect, hex::encode(buffer));
    }
}
