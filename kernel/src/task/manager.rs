use alloc::collections::VecDeque;
use alloc::sync::Arc;

use lazy_static::lazy_static;

use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::task::TaskControlBlock;

pub struct TaskManager {
    queue: VecDeque<Arc<TaskControlBlock>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { queue: VecDeque::new() }
    }

    pub fn add_task(&mut self, task: Arc<TaskControlBlock>) {
        self.queue.push_back(task);
    }

    pub fn fetch_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: SafeCellSingle<TaskManager> = unsafe {
        SafeCellSingle::new(TaskManager::new())
    };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.borrow_exclusive().add_task(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.borrow_exclusive().fetch_task()
}
