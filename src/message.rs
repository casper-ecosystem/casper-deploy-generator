use casper_types::{blake2b, BLAKE2B_DIGEST_LENGTH};

/// It became a de-facto standard in Casper network that messsages for signing
/// are prepended with the following prefix.
const MSG_PREFIX: &str = "Casper Message:\n";

pub(crate) struct CasperMessage(Vec<u8>);

impl CasperMessage {
    /// Create correct instance of `CasperMessage`
    ///
    /// NOTE: It became a de-facto standard that all Casper message for signing
    /// are prepended with `Casper Message:\n`
    pub(crate) fn new(msg: Vec<u8>) -> Self {
        let mut output = MSG_PREFIX.as_bytes().to_vec();
        output.extend(msg);
        CasperMessage(output)
    }

    /// Bypasses the valid header prefix.
    ///
    /// WARNING: Allows for creating invalid instances of `CasperMessage`.
    pub(crate) fn raw(msg: Vec<u8>) -> Self {
        CasperMessage(msg)
    }

    /// Returns reference to the underlying bytes.
    pub(crate) fn inner(&self) -> &[u8] {
        &self.0
    }

    /// Returns blake2b hash of the underlying bytes.
    pub(crate) fn hashed(&self) -> [u8; BLAKE2B_DIGEST_LENGTH] {
        blake2b(&self.0)
    }
}
