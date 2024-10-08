# ndef-rs

`ndef-rs` is a Rust library for working with NFC Data Exchange Format (NDEF) messages. This library provides functionality to parse, create, and manipulate NDEF messages in a convenient and efficient manner.

## Features

- Parse NDEF messages from byte arrays
- Create NDEF messages from scratch
- Support for common NDEF record types (e.g., Text, URI, MIME)
- Easy-to-use API

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ndef-rs = "0.2"
```

## Usage

Here is a simple example of how to use `ndef-rs`:

```rust

use ndef_rs::{NdefMessage, NdefRecord};
use ndef_rs::payload::TextPayload;

fn main() {
  // Create a new NDEF message
  let payload = TextPayload::from_static("Hello, world!");
  let record = NdefRecord::builder()
                  .tnf(TNF::WellKnown)
                  .payload(&payload)
                  .build()
                  .unwrap();
  let ndef_message = NdefMessage::from(&[text_record]);

  // Convert the NDEF message to bytes
  let bytes = ndef_message.to_buffer().unwrap();
  println!("{:?}", bytes);

  // Parse the NDEF message from bytes
  let parsed_message = NdefMessage::decode(&bytes).unwrap();
  println!("{:?}", parsed_message);
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions or suggestions, please open an issue on GitHub.
