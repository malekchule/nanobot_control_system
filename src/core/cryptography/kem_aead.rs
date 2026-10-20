use anyhow::{anyhow, bail, Context, Result};
use pqcrypto::traits::kem::{
    Ciphertext as CiphertextTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait,
    SharedSecret as SharedSecretTrait,
};
use pqcrypto_ntru::ntruhps2048509::{
    ciphertext_bytes, decapsulate, encapsulate, Ciphertext, PublicKey, SecretKey,
};
use ring::rand::SecureRandom;
use ring::{aead, hkdf, rand};

const VERSION: u8 = 1;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const HKDF_SALT: &[u8] = b"nanobot-control-system/aead/v1";
const HKDF_INFO: &[u8] = b"ntru-hps2048509/aes-256-gcm";

fn derive_key(secret: &[u8]) -> Result<aead::LessSafeKey> {
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, HKDF_SALT);
    let prk = salt.extract(secret);

    let info = [HKDF_INFO];
    let okm = prk
        .expand(&info, &aead::AES_256_GCM)
        .map_err(|_| anyhow!("HKDF expansion failed"))?;

    let mut key = [0u8; 32];
    okm.fill(&mut key)
        .map_err(|_| anyhow!("HKDF key extraction failed"))?;

    let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &key)
        .map_err(|_| anyhow!("Invalid AES-256-GCM key"))?;

    Ok(aead::LessSafeKey::new(unbound))
}

pub fn encrypt_data(public_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let pk = PublicKey::from_bytes(public_key).context("Invalid NTRU public key")?;

    let (secret, kem_ct) = encapsulate(&pk);
    let key = derive_key(secret.as_bytes())?;

    let rng = rand::SystemRandom::new();
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill(&mut nonce)
        .map_err(|_| anyhow!("Nonce generation failed"))?;

    let mut output =
        Vec::with_capacity(1 + ciphertext_bytes() + NONCE_LEN + plaintext.len() + TAG_LEN);
    output.push(VERSION);
    output.extend_from_slice(kem_ct.as_bytes());
    output.extend_from_slice(&nonce);

    let mut payload = plaintext.to_vec();

    key.seal_in_place_append_tag(
        aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::from(output.as_slice()),
        &mut payload,
    )
    .map_err(|_| anyhow!("Authenticated encryption failed"))?;

    output.extend_from_slice(&payload);
    Ok(output)
}

pub fn decrypt_data(private_key: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
    let kem_len = ciphertext_bytes();
    let header_len = 1 + kem_len + NONCE_LEN;

    if encrypted.len() < header_len + TAG_LEN {
        bail!("Encrypted message is truncated");
    }

    if encrypted[0] != VERSION {
        bail!("Unsupported encrypted message version");
    }

    let sk = SecretKey::from_bytes(private_key).context("Invalid NTRU private key")?;

    let kem_ct =
        Ciphertext::from_bytes(&encrypted[1..1 + kem_len]).context("Invalid NTRU ciphertext")?;

    let secret = decapsulate(&kem_ct, &sk);
    let key = derive_key(secret.as_bytes())?;

    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&encrypted[1 + kem_len..header_len]);

    let mut payload = encrypted[header_len..].to_vec();

    let plaintext = key
        .open_in_place(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(&encrypted[..header_len]),
            &mut payload,
        )
        .map_err(|_| anyhow!("Ciphertext authentication failed"))?;

    Ok(plaintext.to_vec())
}
