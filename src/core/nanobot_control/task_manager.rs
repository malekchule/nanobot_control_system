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
}
