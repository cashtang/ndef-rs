use std::ops::Deref;

use bitflags::bitflags;
use strum::{VariantArray, FromRepr};

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
);

pub fn get_uri_abbreviation(abbreviation: u8) -> Option<&'static UriAbbrev> {
    URI_ABBREVIATIONS
        .iter()
        .find(|abbr| abbr.0 == abbreviation)
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
