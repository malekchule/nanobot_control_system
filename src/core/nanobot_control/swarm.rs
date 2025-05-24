// Branch: swarm-simulation
// File: src/core/nanobot_control/swarm.rs

use crate::core::cryptography::{generate_qsafe_keypair, sign_data, verify_signature};
use crate::core::nanobot_control::communication::NanobotCommunication;
use crate::core::nanobot_control::task_manager::{TaskManager, NanobotTask};

pub struct NanobotUnit {
    pub id: u32,
    pub comms: NanobotCommunication,
    pub tasks: TaskManager,
}

pub struct NanobotSwarm {
    units: Vec<NanobotUnit>,
}

impl NanobotSwarm {
    pub fn new(size: u32) -> Self {
        let mut units = vec![];
        for id in 0..size {
            let (pubkey, privkey) = generate_qsafe_keypair();
            units.push(NanobotUnit {
                id,
                comms: NanobotCommunication::new(pubkey, privkey),
                tasks: TaskManager::new(),
            });
        }
        NanobotSwarm { units }
    }

    pub fn assign_task_to_unit(&mut self, unit_id: u32, task: NanobotTask) {
        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id) {
            unit.tasks.add_task(task);
        }
    }

    pub fn broadcast_message(&self, message: &[u8]) {
        for unit in &self.units {
            let encrypted = unit.comms.send_secure_message(message);
            let decrypted = unit.comms.receive_secure_message(&encrypted);
            if let Ok(msg) = String::from_utf8(decrypted) {
                println!("Unit {} received: {}", unit.id, msg);
            }
        }
    }

    pub fn execute_all(&mut self) {
        for unit in &mut self.units {
            for task in &unit.tasks.tasks {
                println!("Unit {} executing: {}", unit.id, task.description);
            }
        }
    }
}

