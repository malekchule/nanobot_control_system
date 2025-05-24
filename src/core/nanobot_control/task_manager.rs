use std::fs::{OpenOptions, File};
use std::io::{Write, BufReader, BufRead};
use std::path::Path;
use anyhow::{Result, Context};

pub struct NanobotTask {
    pub task_id: u32,
    pub description: String,
}

pub struct TaskManager {
    tasks: Vec<NanobotTask>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { tasks: vec![] }
    }

    pub fn add_task(&mut self, task: NanobotTask) {
        self.tasks.push(task);
    }

    pub fn execute_task(&self, task_id: u32) {
        if let Some(task) = self.tasks.iter().find(|t| t.task_id == task_id) {
            println!("Executing task: {}", task.description);
        } else {
            println!("Task not found.");
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .context("Unable to open task file for writing")?;

        for task in &self.tasks {
            writeln!(file, "{},{}", task.task_id, task.description)
                .context("Failed to write task to file")?;
        }
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            return Ok(TaskManager { tasks: vec![] });
        }

        let file = File::open(path).context("Unable to open task file for reading")?;
        let reader = BufReader::new(file);

        let mut tasks = vec![];
        for line in reader.lines() {
            let line = line.context("Failed to read line from task file")?;
            let parts: Vec<&str> = line.splitn(2, ',').collect();
            if parts.len() == 2 {
                if let Ok(id) = parts[0].parse::<u32>() {
                    tasks.push(NanobotTask {
                        task_id: id,
                        description: parts[1].to_string(),
                    });
                }
            }
        }
        Ok(TaskManager { tasks })
    }
}

