use std::fmt;

use sha2::{Digest, Sha256};

pub struct LowEntropyHash {
    bits: usize,
    inner: Sha256,
}

impl LowEntropyHash {
    pub fn new(bits: usize) -> Self {
        assert!(bits <= 256);
        assert!(bits > 0);
        Self {
            bits,
            inner: Sha256::new(),
        }
    }

    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        self.inner.update(data);
    }

    pub fn update_chained(mut self, data: impl AsRef<[u8]>) -> Self {
        self.update(data);
        self
    }

    pub fn finalize(self) -> LowEntropyDigest {
        let mut out = [0u8; 32];
        let max_byte_len = (self.bits + 7) / 8;
        out[..max_byte_len].copy_from_slice(&self.inner.finalize().as_slice()[..max_byte_len]);
        let bits_overrun = self.bits % 8;
        out[max_byte_len - 1] &= 0xFFu8 << bits_overrun;
        LowEntropyDigest {
            raw: out,
            bits: self.bits,
        }
    }
}

pub struct LowEntropyDigest {
    raw: [u8; 32],
    bits: usize,
}

impl fmt::Display for LowEntropyDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.raw[..self.bits / 8] {
            write!(f, "{byte:02X}")?;
        }
        let max_byte_len = (self.bits + 7) / 8;

        let bits_overrun = self.bits % 8;
        if bits_overrun > 4 {
            let byte = self.raw[max_byte_len - 1];
            write!(f, "{byte:02X}")?;
        } else if bits_overrun > 0 {
            let byte = self.raw[max_byte_len - 1] >> 4;
            write!(f, "{byte:01X}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_entropy_hash() {
        assert_eq!(
            format!(
                "{}",
                LowEntropyHash::new(23)
                    .update_chained(b"test")
                    .finalize()
                    .to_string()
            ),
            "9F8680".to_string()
        );
    }
}
