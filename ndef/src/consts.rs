use std::ops::Deref;

use bitflags::bitflags;
use strum::{FromRepr, VariantArray};

macro_rules! count_args {
    () => { 0 };
    ($head:expr $(, $tail:expr)*) => { 1 + count_args!($($tail),*) };
}

macro_rules! define_const_array {
    ($arr_name:ident, $elem_type:ty, $(($const_name:ident, $value:expr)),* $(,)?) => {
        $(
            pub const $const_name: $elem_type = $value;
        )*
        pub const $arr_name: [$elem_type; count_args!($($value),*)] = [$($const_name),*];
    };
}

#[derive(Debug, FromRepr, PartialEq, VariantArray, Clone, Copy)]
#[repr(u8)]
pub enum TNF {
    Empty = 0x00,
    WellKnown = 0x01,
    MimeMedia = 0x02,
    AbsoluteUri = 0x03,
    External = 0x04,
    Unknown = 0x05,
    Unchanged = 0x06,
    Reserved = 0x07,
}

pub fn get_tnf_from_repr(repr: u8) -> Option<TNF> {
    TNF::from_repr(repr)
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct UriAbbrev(pub u8, pub &'static str);

impl PartialEq for UriAbbrev {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl UriAbbrev {
    pub fn as_byte(&self) -> u8 {
        self.0
    }

    pub fn as_uri(&self) -> &'static str {
        self.1
    }
}

define_const_array!(
    URI_ABBREVIATIONS,
    UriAbbrev,
    (NONE_ABBRE, UriAbbrev(0x00, "")),
    (HTTP_WWW, UriAbbrev(0x01, "http://www.")),
    (HTTPS_WWW, UriAbbrev(0x02, "https://www.")),
    (HTTP, UriAbbrev(0x03, "http://")),
    (HTTPS, UriAbbrev(0x04, "https://")),
    (TEL, UriAbbrev(0x05, "tel:")),
    (MAILTO, UriAbbrev(0x06, "mailto:")),
    (FTP_ANONYMOUS, UriAbbrev(0x07, "ftp://anonymous:anonymous@")),
    (FTP_FTP, UriAbbrev(0x08, "ftp://ftp.")),
    (FTPS, UriAbbrev(0x09, "ftps://")),
    (SFTP, UriAbbrev(0x0A, "sftp://")),
    (SMB, UriAbbrev(0x0B, "smb://")),
    (NFS, UriAbbrev(0x0C, "nfs://")),
    (FTP, UriAbbrev(0x0D, "ftp://")),
    (DAV, UriAbbrev(0x0E, "dav://")),
    (NEWS, UriAbbrev(0x0F, "news:")),
    (TELNET, UriAbbrev(0x10, "telnet://")),
    (IMAP, UriAbbrev(0x11, "imap:")),
    (RTSP, UriAbbrev(0x12, "rtsp://")),
    (URN, UriAbbrev(0x13, "urn:")),
    (POP, UriAbbrev(0x14, "pop:")),
    (SIP, UriAbbrev(0x15, "sip:")),
    (SIPS, UriAbbrev(0x16, "sips:")),
    (TFTP, UriAbbrev(0x17, "tftp:")),
    (BTSPP, UriAbbrev(0x18, "btspp://")),
    (BTL2CAP, UriAbbrev(0x19, "btl2cap://")),
    (BTGOEP, UriAbbrev(0x1A, "btgoep://")),
    (TCPOBEX, UriAbbrev(0x1B, "tcpobex://")),
    (IRDAOBEX, UriAbbrev(0x1C, "irdaobex://")),
    (FILE, UriAbbrev(0x1D, "file://")),
    (URN_EPC_ID, UriAbbrev(0x1E, "urn:epc:id:")),
    (URN_EPC_TAG, UriAbbrev(0x1F, "urn:epc:tag:")),
    (URN_EPC_PAT, UriAbbrev(0x20, "urn:epc:pat:")),
    (URN_EPC_RAW, UriAbbrev(0x21, "urn:epc:raw:")),
    (URN_EPC, UriAbbrev(0x22, "urn:epc:")),
    (URN_NFC, UriAbbrev(0x23, "urn:nfc:")),
);

pub fn get_uri_abbreviation(abbreviation: u8) -> Option<&'static UriAbbrev> {
    URI_ABBREVIATIONS.iter().find(|abbr| abbr.0 == abbreviation)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RTD(pub &'static [u8]);

define_const_array!(
    RTD_PRE_DEFINED,
    RTD,
    (RTD_TEXT, RTD(b"T")),
    (RTD_URI, RTD(b"U")),
    (RTD_SMART_POSTER, RTD(b"Sp")),
);

impl RTD {
    pub fn as_bytes(&self) -> &'static [u8] {
        self.0
    }
}

impl From<RTD> for Vec<u8> {
    fn from(record_type: RTD) -> Self {
        record_type.0.to_vec()
    }
}

impl PartialEq<[u8]> for RTD {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

impl PartialEq<Vec<u8>> for RTD {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == other.as_slice()
    }
}

impl Deref for RTD {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct RecordFlags: u8 {
        const MB = 0b1000_0000;
        const ME = 0b0100_0000;
        const CF = 0b0010_0000;
        const SR = 0b0001_0000;
        const IL = 0b0000_1000;
        const TNF = 0b0000_0111;
    }
}
