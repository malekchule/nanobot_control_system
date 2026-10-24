use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct NanobotTask {
    pub task_id: u32,
    pub description: String,
}

pub struct TaskManager {
    tasks: Vec<NanobotTask>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { tasks: vec![] }
    }

    pub fn add_task(&mut self, task: NanobotTask) {
        self.tasks.push(task);
    }

    pub fn iter_tasks(&self) -> impl Iterator<Item = &NanobotTask> {
        self.tasks.iter()
    }

    pub fn execute_task(&self, task_id: u32) {
        if let Some(task) = self.tasks.iter().find(|t| t.task_id == task_id) {
            println!("Executing task: {}", task.description);
        } else {
            println!("Task not found.");
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        for task in &self.tasks {
            if task.description.contains('\n') || task.description.contains('\r') {
                anyhow::bail!("Task {} description contains a newline", task.task_id);
            }
        }

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
        let mut seen_ids = HashSet::new();
        for (index, line) in reader.lines().enumerate() {
            let line_number = index + 1;
            let line =
                line.with_context(|| format!("Failed to read task file at line {line_number}"))?;
            let (raw_id, description) = line.split_once(',').ok_or_else(|| {
                anyhow!("Invalid task record at line {line_number}: missing comma")
            })?;
            let id = raw_id
                .parse::<u32>()
                .with_context(|| format!("Invalid task ID at line {line_number}: {raw_id:?}"))?;
            if !seen_ids.insert(id) {
                anyhow::bail!("Duplicate task ID {id} at line {line_number}");
            }

            tasks.push(NanobotTask {
                task_id: id,
                description: description.to_string(),
            });
        }
        Ok(TaskManager { tasks })
    }
}

#[cfg(test)]
mod file_validation_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn with_task_file(content: &str, test: impl FnOnce(&str)) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let path =
            std::env::temp_dir().join(format!("nanobot-tasks-{}-{nonce}.txt", std::process::id()));

        std::fs::write(&path, content).unwrap();

        let path_string = path.to_string_lossy().into_owned();
        test(&path_string);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_multiline_descriptions_without_overwriting_file() {
        for separator in ["\n", "\r"] {
            with_task_file("7,existing task\n", |path| {
                let mut manager = TaskManager::new();
                manager.add_task(NanobotTask {
                    task_id: 42,
                    description: format!("first{separator}second"),
                });

                let error = manager.save_to_file(path).unwrap_err();

                assert!(error.to_string().contains("Task 42"));
                assert!(error.to_string().contains("newline"));
                assert_eq!(std::fs::read_to_string(path).unwrap(), "7,existing task\n");
            });
        }
    }

    #[test]
    fn rejects_missing_separator_with_line_number() {
        with_task_file("1,valid\nmalformed\n", |path| {
            let result = TaskManager::load_from_file(path);
            let error = result.err().expect("expected malformed record error");

            assert!(error.to_string().contains("line 2"));
            assert!(error.to_string().contains("missing comma"));
        });
    }

    #[test]
    fn rejects_duplicate_task_ids_with_line_number() {
        with_task_file("42,first task\n42,second task\n", |path| {
            let error = TaskManager::load_from_file(path)
                .err()
                .expect("expected duplicate task ID error");

            assert!(error.to_string().contains("Duplicate task ID 42"));
            assert!(error.to_string().contains("line 2"));
        });
    }

    #[test]
    fn accepts_distinct_task_ids() {
        with_task_file("42,first task\n43,second task\n", |path| {
            let manager = TaskManager::load_from_file(path).unwrap();
            let ids: Vec<_> = manager.iter_tasks().map(|task| task.task_id).collect();

            assert_eq!(ids, vec![42, 43]);
        });
    }

    #[test]
    fn rejects_invalid_task_id_with_line_number() {
        with_task_file("abc,invalid\n", |path| {
            let result = TaskManager::load_from_file(path);
            let error = result.err().expect("expected invalid ID error");

            assert!(error.to_string().contains("line 1"));
            assert!(error.to_string().contains("Invalid task ID"));
        });
    }

    #[test]
    fn preserves_commas_in_valid_task_descriptions() {
        with_task_file("42,Inspect, repair, report\n", |path| {
            let manager = TaskManager::load_from_file(path).unwrap();
            let tasks: Vec<_> = manager.iter_tasks().collect();

            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].task_id, 42);
            assert_eq!(tasks[0].description, "Inspect, repair, report");
        });
    }
}
