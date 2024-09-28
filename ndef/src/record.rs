
pub struct NdefRecord {
    /// The Type Name Format (TNF) field of the record.
    tnf: TNF,
    /// The type field of the record.
    type_: Vec<u8>,
    /// The ID field of the record.
    id: Vec<u8>,
    /// The payload field of the record.
    payload: Vec<u8>,
}