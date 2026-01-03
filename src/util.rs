use std::io::{Read, Result};

use encoding_rs::{Encoding, SHIFT_JIS};

#[derive(Debug)]
pub struct DecodedTextResult<'c> {
    text: String,
    encoding: &'c Encoding,
    had_errors: bool,
}

impl<'c> DecodedTextResult<'c> {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn encoding(&self) -> &'c Encoding {
        self.encoding
    }

    pub fn had_errors(&self) -> bool {
        self.had_errors
    }
}

pub fn read_u32_le<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_u8<R: Read>(r: &mut R) -> Result<u8> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

pub fn decode_text(bytes: &[u8]) -> DecodedTextResult<'_> {
    let (cow, encoding, had_errors) = SHIFT_JIS.decode(bytes);

    DecodedTextResult {
        text: cow.to_string(),
        encoding,
        had_errors,
    }
}
