// Branch: add-tests
// File: tests/integration_tests.rs

use nanobot_control_system::core::cryptography::*;
use nanobot_control_system::core::nanobot_control::communication::NanobotCommunication;
use nanobot_control_system::core::nanobot_control::task_manager::{TaskManager, NanobotTask};

#[test]
fn test_keypair_sign_verify() {
    let (public, private) = generate_qsafe_keypair();
    let message = b"Verify me";
    let sig = sign_data(&private, message);
    assert!(verify_signature(&public, message, &sig));
}

#[test]
fn test_encryption_decryption_roundtrip() {
    let (public, private) = generate_qsafe_keypair();
    let message = b"Secret Message";
    let encrypted = encrypt_data(&public, message);
    let decrypted = decrypt_data(&private, &encrypted);
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
    let encrypted = comm.send_secure_message(msg);
    let first = comm.receive_secure_message(&encrypted);
    let second = comm.receive_secure_message(&encrypted);

    assert_eq!(first, b"Payload");
    assert_eq!(second, b"INVALID: Replay detected");
}

