use crate::{record::NdefRecord, *};
use anyhow::{bail, Result};
use std::io::Cursor;

#[derive(Default, Debug)]
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
            if record.flags() & RecordFlags::MB == RecordFlags::MB && !records.is_empty() {
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
    use crate::payload::*;
    use crate::record::NdefRecord;
    use crate::*;

    #[test]
    fn test_multiple_records() {
        let record1 = NdefRecord::builder()
            .tnf(TNF::WellKnown)
            .payload(&UriPayload::from_static("weixin://dl/business"))
            .build()
            .unwrap();

        let record2 = NdefRecord::builder()
            .tnf(TNF::External)
            .payload(&ExternalPayload::from_static(
                b"android.com:pkg",
                b"com.tencent.mm",
            ))
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
        let payload = UriPayload::try_from(record).unwrap();
        assert_eq!(NONE_ABBRE, payload.abbreviation());
        assert_eq!("weixin://dl/business", payload.uri());
        assert_eq!("weixin://dl/business", payload.full_uri());

        let record = message.records().get(1).unwrap();
        assert_eq!(TNF::External, record.tnf());
        assert_eq!(b"android.com:pkg", record.record_type());
        assert_eq!(b"com.tencent.mm", record.payload());

        assert!(UriPayload::try_from(record).is_err());
    }

    #[test]
    fn test_single_record() {
        let record = NdefRecord::builder()
            .tnf(TNF::WellKnown)
            .payload(&UriPayload::with_abbrev(
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
        assert_eq!(TNF::WellKnown, record.tnf());
        assert_eq!(RTD_URI.as_bytes(), record.record_type());
        let payload = UriPayload::try_from(record).unwrap();
        assert_eq!(HTTP_WWW, payload.abbreviation());
        assert_eq!("supwisdom.com", payload.uri());
        assert_eq!("http://www.supwisdom.com", payload.full_uri());
    }

    #[test]
    fn test_not_sr() {
        let record = NdefRecord::builder()
            .tnf(TNF::External)
            .payload(&SmartPosterPayload::from_static(&[0xabu8; 300]))
            .build()
            .unwrap();
        let message = NdefMessage::from(record);
        let buffer = message.to_buffer().unwrap();
        let expect = "c4022c0100005370abababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababab";
        assert_eq!(expect, hex::encode(buffer));
    }

    #[test]
    fn test_complex_structure_decode() {
        let expect = "9c1e550469736f2e6f72673a31383031333a646576696365656e676167656d656e746d646f63d8185851a30063312e30018201d8185828a3010120042158207e5c55b2acd1cce87fe9dbcba205afe165ad7261930d5df7b1bbce7a5cd9c1430281830201a300f501f40a50fc52fb75fe8a431eaf7d34b39cbba8f8110211487315d1020b61630103424c4501046d646f635a2015036170706c69636174696f6e2f766e642e626c7565746f6f74682e6c652e6f6f62424c45021c001107f8a8bb9cb3347daf1e438afe75fb52fc";
        let message = NdefMessage::decode(hex::decode(expect).unwrap()).unwrap();
        assert_eq!(3, message.records().len());
    }
}
