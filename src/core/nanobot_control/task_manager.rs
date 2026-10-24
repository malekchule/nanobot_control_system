use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::io::{BufRead, BufReader, Write};

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

        let mut seen_ids = std::collections::HashSet::new();
        for task in &self.tasks {
            if !seen_ids.insert(task.task_id) {
                anyhow::bail!("Duplicate task ID {} cannot be saved", task.task_id);
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

        file.flush().context("Failed to flush task file")?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(TaskManager::new());
            }
            Err(error) => {
                return Err(error).context("Unable to open task file for reading");
            }
        };
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
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TempTaskFile(std::path::PathBuf);

    impl Drop for TempTaskFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn with_task_file(content: &str, test: impl FnOnce(&str)) {
        let (path, mut file) = loop {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("nanobot-tasks-{}-{id}.txt", std::process::id()));

            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => break (path, file),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    continue;
                }
                Err(error) => panic!("Failed to create task test file: {error}"),
            }
        };

        file.write_all(content.as_bytes()).unwrap();
        drop(file);

        let fixture = TempTaskFile(path);
        let path_string = fixture.0.to_string_lossy().into_owned();
        test(&path_string);
    }

    #[test]
    fn cleans_up_task_file_after_test_panic() {
        let captured_path = std::cell::RefCell::new(None);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_task_file("42,fixture\\n", |path| {
                *captured_path.borrow_mut() = Some(path.to_string());

                panic!("intentional test panic");
            });
        }));

        assert!(result.is_err());

        let path = captured_path
            .into_inner()
            .expect("fixture path should be captured");

        assert!(
            !std::path::Path::new(&path).exists(),
            "temporary task file was not removed"
        );
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
    fn rejects_duplicate_ids_without_overwriting_file() {
        with_task_file("7,existing task\n", |path| {
            let mut manager = TaskManager::new();

            for description in ["first", "second"] {
                manager.add_task(NanobotTask {
                    task_id: 42,
                    description: description.to_string(),
                });
            }

            let error = manager.save_to_file(path).unwrap_err();

            assert!(error.to_string().contains("Duplicate task ID 42"));
            assert_eq!(std::fs::read_to_string(path).unwrap(), "7,existing task\n");
        });
    }

    #[test]
    fn saved_tasks_can_be_reloaded_immediately() {
        with_task_file("", |path| {
            let mut manager = TaskManager::new();

            manager.add_task(NanobotTask {
                task_id: 101,
                description: "Inspect, verify, report".to_string(),
            });

            manager.save_to_file(path).unwrap();

            let restored = TaskManager::load_from_file(path).unwrap();
            let tasks: Vec<_> = restored.iter_tasks().collect();

            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].task_id, 101);
            assert_eq!(tasks[0].description, "Inspect, verify, report");
        });
    }

    #[test]
    fn saves_distinct_task_ids() {
        with_task_file("", |path| {
            let mut manager = TaskManager::new();

            for id in [42, 43] {
                manager.add_task(NanobotTask {
                    task_id: id,
                    description: format!("task {id}"),
                });
            }

            manager.save_to_file(path).unwrap();

            assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                "42,task 42\n43,task 43\n"
            );
        });
    }

    #[test]
    fn missing_task_file_returns_empty_manager() {
        with_task_file("", |path| {
            std::fs::remove_file(path).unwrap();

            let manager = TaskManager::load_from_file(path).unwrap();

            assert_eq!(manager.iter_tasks().count(), 0);

            // Restore the fixture for the test helper's cleanup.
            std::fs::write(path, "").unwrap();
        });
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
