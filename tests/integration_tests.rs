// Branch: add-tests
// File: tests/integration_tests.rs

use nanobot_control_system::core::cryptography::*;
use nanobot_control_system::core::nanobot_control::communication::NanobotCommunication;
use nanobot_control_system::core::nanobot_control::task_manager::{NanobotTask, TaskManager};

#[test]
fn test_keypair_sign_verify() {
    let (public, private) = generate_signing_keypair();
    let message = b"Verify me";

    let signature = sign_data(&private, message).unwrap();

    assert!(verify_signature(&public, message, &signature));
    assert!(!verify_signature(&public, b"Modified message", &signature));

    let mut tampered_signature = signature.clone();
    tampered_signature[0] ^= 1;

    assert!(!verify_signature(&public, message, &tampered_signature));

    assert!(!verify_signature(&public, message, &[]));
    assert!(!verify_signature(&[], message, &signature));
}

#[test]
fn test_encryption_decryption_roundtrip() {
    let (public, private) = generate_qsafe_keypair();
    let message = b"Secret Message";
    let encrypted = encrypt_data(&public, message).unwrap();
    let decrypted = decrypt_data(&private, &encrypted).unwrap();
    assert_eq!(decrypted, message);
}

#[test]
fn test_task_manager_execution() {
    let mut manager = TaskManager::new();
    manager.add_task(NanobotTask {
        task_id: 42,
        description: "Deliver vaccine".to_string(),
    });
    manager.execute_task(42); // This should not panic
}

#[test]
fn test_secure_message_with_replay_protection() {
    let (public, private) = generate_qsafe_keypair();
    let comm = NanobotCommunication::new(public.clone(), private.clone());

    let msg = b"Payload";
    let encrypted = comm.send_secure_message(msg).unwrap();
    let first = comm.receive_secure_message(&encrypted).unwrap();
    let second = comm.receive_secure_message(&encrypted);

    assert_eq!(first, b"Payload");
    assert!(second.unwrap_err().to_string().contains("Replay detected"));
}

#[test]
fn test_aead_rejects_tampering() {
    let (public, private) = generate_qsafe_keypair();
    let encrypted = encrypt_data(&public, b"Important command").unwrap();

    // Every byte is authenticated, including the header.
    for index in [0, 1, encrypted.len() - 1] {
        let mut modified = encrypted.clone();
        modified[index] ^= 1;

        assert!(
            decrypt_data(&private, &modified).is_err(),
            "Tampering at offset {index} was accepted"
        );
    }
}

#[test]
fn test_aead_rejects_wrong_key() {
    let (public, _) = generate_qsafe_keypair();
    let (_, wrong_private) = generate_qsafe_keypair();

    let encrypted = encrypt_data(&public, b"Secret").unwrap();

    assert!(decrypt_data(&wrong_private, &encrypted).is_err());
}

#[test]
fn test_aead_rejects_truncated_ciphertext() {
    let (public, private) = generate_qsafe_keypair();
    let encrypted = encrypt_data(&public, b"Secret").unwrap();

    for length in [0, 1, 12, encrypted.len() - 1] {
        assert!(decrypt_data(&private, &encrypted[..length]).is_err());
    }
}

#[test]
fn test_aead_rejects_invalid_keys() {
    let (public, private) = generate_qsafe_keypair();

    assert!(encrypt_data(&[], b"Secret").is_err());

    let encrypted = encrypt_data(&public, b"Secret").unwrap();

    assert!(decrypt_data(&[], &encrypted).is_err());
    assert!(decrypt_data(&private[..8], &encrypted).is_err());
}

#[test]
fn test_aead_encryptions_are_distinct() {
    let (public, private) = generate_qsafe_keypair();

    let first = encrypt_data(&public, b"Same message").unwrap();
    let second = encrypt_data(&public, b"Same message").unwrap();

    assert_ne!(first, second);
    assert_eq!(decrypt_data(&private, &first).unwrap(), b"Same message");
    assert_eq!(decrypt_data(&private, &second).unwrap(), b"Same message");
}

#[test]
fn test_aead_empty_message_roundtrip() {
    let (public, private) = generate_qsafe_keypair();

    let encrypted = encrypt_data(&public, b"").unwrap();
    assert_eq!(decrypt_data(&private, &encrypted).unwrap(), b"");
}

#[test]
fn test_failed_authentication_does_not_consume_replay_identifier() {
    let (public, private) = generate_qsafe_keypair();
    let comm = NanobotCommunication::new(public, private);

    let encrypted = comm.send_secure_message(b"Payload").unwrap();

    let mut tampered = encrypted.clone();
    *tampered.last_mut().unwrap() ^= 1;

    assert!(comm.receive_secure_message(&tampered).is_err());

    // The authentic message must still be accepted.
    assert_eq!(comm.receive_secure_message(&encrypted).unwrap(), b"Payload");

    // A genuine replay must still be rejected.
    assert!(comm.receive_secure_message(&encrypted).is_err());
}

#[test]
fn test_aead_rejects_nonce_tampering() {
    use pqcrypto_ntru::ntruhps2048509::ciphertext_bytes;

    let (public, private) = generate_qsafe_keypair();
    let encrypted = encrypt_data(&public, b"Authenticated message").unwrap();

    let nonce_offset = 1 + ciphertext_bytes();

    let mut tampered = encrypted;
    tampered[nonce_offset] ^= 1;

    assert!(decrypt_data(&private, &tampered).is_err());
}
