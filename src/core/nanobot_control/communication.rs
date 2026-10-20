use crate::core::cryptography::{decrypt_data, encrypt_data};
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct NanobotCommunication {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    used_nonces: Arc<Mutex<HashSet<Uuid>>>,
}

impl NanobotCommunication {
    pub fn new(public_key: Vec<u8>, private_key: Vec<u8>) -> Self {
        Self {
            public_key,
            private_key,
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn send_secure_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        let nonce = Uuid::new_v4().to_string();

        let mut payload = nonce.into_bytes();
        payload.extend_from_slice(message);

        encrypt_data(&self.public_key, &payload)
    }

    pub fn receive_secure_message(&self, encrypted_message: &[u8]) -> Result<Vec<u8>> {
        let decrypted = decrypt_data(&self.private_key, encrypted_message)?;

        if decrypted.len() < 36 {
            bail!("Missing message identifier");
        }

        let (identifier, message) = decrypted.split_at(36);

        let identifier =
            std::str::from_utf8(identifier).context("Invalid message identifier encoding")?;

        let nonce = Uuid::parse_str(identifier).context("Invalid message identifier")?;

        let mut seen = self
            .used_nonces
            .lock()
            .map_err(|_| anyhow::anyhow!("Replay cache lock poisoned"))?;

        if !seen.insert(nonce) {
            bail!("Replay detected");
        }

        Ok(message.to_vec())
    }
}
