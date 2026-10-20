use pqcrypto::traits::kem::{
    Ciphertext as CiphertextTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait,
    SharedSecret as SharedSecretTrait,
};
use pqcrypto_ntru::ntruhps2048509::*;

pub fn generate_qsafe_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}

pub fn encrypt_data(public_key: &[u8], data: &[u8]) -> Vec<u8> {
    let pk = PublicKey::from_bytes(public_key).expect("Invalid public key");
    let (shared_secret, ciphertext) = encapsulate(&pk);

    data.iter()
        .zip(shared_secret.as_bytes().iter().cycle())
        .map(|(d, k)| d ^ k)
        .chain(ciphertext.as_bytes().iter().copied())
        .collect()
}

pub fn decrypt_data(private_key: &[u8], encrypted: &[u8]) -> Vec<u8> {
    let ct_len = ciphertext_bytes();
    let split_index = encrypted.len().saturating_sub(ct_len);
    let (data_encrypted, ct) = encrypted.split_at(split_index);

    let sk = SecretKey::from_bytes(private_key).expect("Invalid private key");
    let ct = Ciphertext::from_bytes(ct).expect("Invalid ciphertext");
    let shared_secret = decapsulate(&ct, &sk);

    data_encrypted
        .iter()
        .zip(shared_secret.as_bytes().iter().cycle())
        .map(|(d, k)| d ^ k)
        .collect()
}
