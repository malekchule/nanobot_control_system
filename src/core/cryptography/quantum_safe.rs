use pqcrypto::traits::kem::{PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use pqcrypto_ntru::ntruhps2048509::keypair;

pub fn generate_qsafe_keypair() -> (Vec<u8>, Vec<u8>) {
    let (pk, sk) = keypair();
    (pk.as_bytes().to_vec(), sk.as_bytes().to_vec())
}
