

/// The `TNF` enum represents the Type Name Format (TNF) field in an NDEF record.
/// Each variant corresponds to a specific TNF value as defined by the NDEF specification.
///
/// Variants:
/// - `Empty`: Indicates an empty record (0x00).
/// - `WellKnown`: Indicates a well-known record type (0x01).
/// - `MimeMedia`: Indicates a MIME media record type (0x02).
/// - `AbsoluteUri`: Indicates an absolute URI record type (0x03).
/// - `External`: Indicates an external record type (0x04).
/// - `Unknown`: Indicates an unknown record type (0x05).
/// - `Unchanged`: Indicates an unchanged record type (0x06).
/// - `Reserved`: Reserved for future use (0x07).

/// The `UriAbbreviation` enum represents abbreviations for URIs in an NDEF record.
/// Each variant corresponds to a specific URI abbreviation as defined by the NDEF specification.
///
/// Variants:
/// - `None`: No abbreviation (0x00).
/// - `HttpWww`: Abbreviation for "http://www." (0x01).
/// - `HttpsWww`: Abbreviation for "https://www." (0x02).
/// - `Http`: Abbreviation for "http://" (0x03).
/// - `Https`: Abbreviation for "https://" (0x04).
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

pub enum UriAbbreviation {
    None = 0x00,
    HttpWww = 0x01,
    HttpsWww = 0x02,
    Http = 0x03,
    Https = 0x04,
}