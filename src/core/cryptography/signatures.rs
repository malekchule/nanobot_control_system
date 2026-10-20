use anyhow::{Context, Result};
use pqcrypto::traits::sign::{
    DetachedSignature as DetachedSignatureTrait, PublicKey as PublicKeyTrait,
    SecretKey as SecretKeyTrait,
};
use pqcrypto_mldsa::mldsa65;

pub fn generate_signing_keypair() -> (Vec<u8>, Vec<u8>) {
    let (public_key, secret_key) = mldsa65::keypair();

    (
        public_key.as_bytes().to_vec(),
        secret_key.as_bytes().to_vec(),
    )
}

pub fn sign_data(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    let secret_key =
        mldsa65::SecretKey::from_bytes(secret_key).context("Invalid ML-DSA-65 secret key")?;

    Ok(mldsa65::detached_sign(message, &secret_key)
        .as_bytes()
        .to_vec())
}

pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let Ok(public_key) = mldsa65::PublicKey::from_bytes(public_key) else {
        return false;
    };

    let Ok(signature) = mldsa65::DetachedSignature::from_bytes(signature) else {
        return false;
    };

    mldsa65::verify_detached_signature(&signature, message, &public_key).is_ok()
}
