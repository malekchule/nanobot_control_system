mod core;
mod metrics;

use core::cryptography::{generate_qsafe_keypair, sign_data, verify_signature};
use core::nanobot_control::communication::NanobotCommunication;
use core::nanobot_control::task_manager::{TaskManager, NanobotTask};
use metrics::{TASKS_EXECUTED, ACTIVE_UNITS, serve_metrics};
use log::{info, error};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let metrics_addr: SocketAddr = "127.0.0.1:9898".parse()?;
    task::spawn(async move {
        serve_metrics(metrics_addr).await;
    });
    info!("Metrics server running on http://{}/metrics", metrics_addr);

    let (public_key, private_key) = generate_qsafe_keypair();
    info!("Generated quantum-safe keypair.");

    let nanobot_comm = NanobotCommunication::new(public_key.clone(), private_key.clone());
    info!("Nanobot communication interface initialized.");

    let message = b"Critical medical task: Deliver medication.";
    let encrypted_message = nanobot_comm.send_secure_message(message);
    info!("Encrypted message sent to nanobot.");

    let decrypted_message = nanobot_comm.receive_secure_message(&encrypted_message);
    let msg_str = String::from_utf8(decrypted_message)
        .map_err(|e| anyhow::anyhow!("UTF-8 conversion error: {}", e))?;
    info!("Decrypted message: {}", msg_str);

    let mut task_manager = TaskManager::new();
    task_manager.add_task(NanobotTask {
        task_id: 1,
        description: "Deliver medication to target cells.".to_string(),
    });
    task_manager.execute_task(1);
    TASKS_EXECUTED.inc();

    let signature = sign_data(&private_key, message);
    let is_valid = verify_signature(&public_key, message, &signature);
    info!("Signature verification result: {}", is_valid);

    ACTIVE_UNITS.set(1);

    Ok(())
}

