// Branch: swarm-simulation
// File: src/core/nanobot_control/swarm.rs

use crate::core::cryptography::generate_qsafe_keypair;
use crate::core::nanobot_control::communication::NanobotCommunication;
use crate::core::nanobot_control::task_manager::{NanobotTask, TaskManager};
use anyhow::Result;

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

    pub fn assign_task_to_unit(&mut self, unit_id: u32, task: NanobotTask) -> Result<()> {
        let unit = self
            .units
            .iter_mut()
            .find(|unit| unit.id == unit_id)
            .ok_or_else(|| anyhow::anyhow!("Nanobot unit {unit_id} not found"))?;

        unit.tasks.add_task(task);
        Ok(())
    }

    pub fn broadcast_message(&self, message: &[u8]) -> Result<()> {
        for unit in &self.units {
            let encrypted = unit.comms.send_secure_message(message)?;
            let decrypted = unit.comms.receive_secure_message(&encrypted)?;

            let msg = String::from_utf8(decrypted)?;
            println!("Unit {} received: {}", unit.id, msg);
        }

        Ok(())
    }

    pub fn execute_all(&mut self) {
        for unit in &mut self.units {
            for task in unit.tasks.iter_tasks() {
                println!("Unit {} executing: {}", unit.id, task.description);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_task_to_existing_unit() {
        let mut swarm = NanobotSwarm::new(1);

        swarm
            .assign_task_to_unit(
                0,
                NanobotTask {
                    task_id: 42,
                    description: "Inspect unit".to_string(),
                },
            )
            .unwrap();

        let tasks: Vec<_> = swarm.units[0].tasks.iter_tasks().collect();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_id, 42);
        assert_eq!(tasks[0].description, "Inspect unit");
    }

    #[test]
    fn rejects_assignment_to_unknown_unit() {
        let mut swarm = NanobotSwarm::new(1);

        let result = swarm.assign_task_to_unit(
            99,
            NanobotTask {
                task_id: 42,
                description: "Inspect unit".to_string(),
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("99"));
        assert_eq!(swarm.units[0].tasks.iter_tasks().count(), 0);
    }
}
