use smol_str::SmolStr;
use xdid::core::did::Did;

use crate::RecordId;

/// Signed batch of Loro operations.
#[derive(Clone, Debug)]
pub struct Envelope {
    /// Target record ID.
    pub record_id: RecordId,
    /// Loro-encoded updates (`ExportMode::updates`).
    pub ops: Vec<u8>,
    /// Version vector this update builds on (from Loro).
    pub from_version: Vec<u8>,
    /// Version vector after applying ops.
    pub to_version: Vec<u8>,
    /// Author's DID.
    pub author: Did,
    /// Signature over canonical encoding.
    pub signature: Signature,
}

impl Envelope {
    /// Canonical bytes for signing/verification.
    ///
    /// # Panics
    ///
    /// Panics if `ops`, `from_version`, or `to_version` length exceeds `u32::MAX`.
    #[must_use]
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.record_id.as_str().as_bytes());
        buf.extend(
            &u32::try_from(self.ops.len())
                .expect("ops length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.ops);
        buf.extend(
            &u32::try_from(self.from_version.len())
                .expect("from_version length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.from_version);
        buf.extend(
            &u32::try_from(self.to_version.len())
                .expect("to_version length exceeds u32::MAX")
                .to_be_bytes(),
        );
        buf.extend(&self.to_version);
        buf.extend(self.author.to_string().as_bytes());
        buf
    }
}

/// Cryptographic signature.
#[derive(Clone, Debug)]
pub struct Signature {
    /// Algorithm identifier (e.g., "ES256" for P-256).
    pub alg: SmolStr,
    /// Raw signature bytes (64 bytes for P-256).
    pub bytes: Vec<u8>,
}
