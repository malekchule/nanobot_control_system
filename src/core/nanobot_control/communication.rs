use crate::core::cryptography::{encrypt_data, decrypt_data};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct NanobotCommunication {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    used_nonces: Arc<Mutex<HashSet<String>>>,
}

impl NanobotCommunication {
    pub fn new(public_key: Vec<u8>, private_key: Vec<u8>) -> Self {
        NanobotCommunication {
            public_key,
            private_key,
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn send_secure_message(&self, message: &[u8]) -> Vec<u8> {
        let nonce = Uuid::new_v4().to_string();
        let mut payload = nonce.as_bytes().to_vec();
        payload.extend_from_slice(message);
        encrypt_data(&self.public_key, &payload)
    }

    pub fn receive_secure_message(&self, encrypted_message: &[u8]) -> Vec<u8> {
        let decrypted = decrypt_data(&self.private_key, encrypted_message);

        if decrypted.len() < 36 {
            return b"INVALID: Missing nonce".to_vec();
        }

        let (nonce_bytes, message) = decrypted.split_at(36);
        let nonce = String::from_utf8_lossy(nonce_bytes).to_string();

        let mut seen = self.used_nonces.lock().unwrap();
        if seen.contains(&nonce) {
            return b"INVALID: Replay detected".to_vec();
        }
        seen.insert(nonce);

        message.to_vec()
    }
}

