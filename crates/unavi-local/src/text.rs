//! Bytes as local-storage text: hex, read back with a raw fallback.
//!
//! A value that is not hex reads back as its raw text. Values stored raw by an
//! earlier build stay readable through it; it can go once those have been
//! rewritten.

pub fn encode(value: &[u8]) -> String {
    hex::encode(value)
}

pub fn decode(text: &str) -> Vec<u8> {
    hex::decode(text).unwrap_or_else(|_| text.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_round_trip() {
        for value in [&b""[..], b"text", &[0xFF, 0x00, 0x7F]] {
            assert_eq!(decode(&encode(value)), value);
        }
    }

    #[test]
    fn text_round_trips_through_bytes() {
        let stored = encode(b"[keybinds]\njump = \"Space\"");
        assert_eq!(
            String::from_utf8(decode(&stored)).expect("utf8"),
            "[keybinds]\njump = \"Space\""
        );
    }

    #[test]
    fn a_raw_value_reads_back_as_itself() {
        let pem = "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----\n";
        assert_eq!(decode(pem), pem.as_bytes());
    }
}
