// src/main.rs

mod core;

use core::cryptography::{generate_qsafe_keypair, sign_data, verify_signature};
use core::nanobot_control::communication::NanobotCommunication;
use core::nanobot_control::task_manager::{TaskManager, NanobotTask};

fn main() {
    let (public_key, private_key) = generate_qsafe_keypair();

    let nanobot_comm = NanobotCommunication::new(public_key.clone(), private_key.clone());

    let message = b"Critical medical task: Deliver medication.";
    let encrypted_message = nanobot_comm.send_secure_message(message);
    let decrypted_message = nanobot_comm.receive_secure_message(&encrypted_message);

    println!("Decrypted message: {:?}", String::from_utf8(decrypted_message).unwrap());

    let mut task_manager = TaskManager::new();
    task_manager.add_task(NanobotTask {
        task_id: 1,
        description: "Deliver medication to target cells.".to_string(),
    });
    task_manager.execute_task(1);

    let signature = sign_data(&private_key, message);
    let is_valid = verify_signature(&public_key, message, &signature);

    println!("Signature verification result: {}", is_valid);
}
