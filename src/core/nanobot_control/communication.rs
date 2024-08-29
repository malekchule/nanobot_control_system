use crate::core::cryptography::{encrypt_data, decrypt_data};

pub struct NanobotCommunication {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

impl NanobotCommunication {
    pub fn new(public_key: Vec<u8>, private_key: Vec<u8>) -> Self {
        NanobotCommunication { public_key, private_key }
    }

    pub fn send_secure_message(&self, message: &[u8]) -> Vec<u8> {
        encrypt_data(&self.public_key, message)
    }

    pub fn receive_secure_message(&self, encrypted_message: &[u8]) -> Vec<u8> {
        decrypt_data(&self.private_key, encrypted_message)
    }
}
