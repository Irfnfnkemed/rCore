use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::mem::take;
use core::usize::MAX;

use lazy_static::lazy_static;

use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::task::TaskControlBlock;

pub struct TaskManager {
    queue: VecDeque<Arc<TaskControlBlock>>,
    server: BTreeMap<isize, Arc<TaskControlBlock>>,
    server_status: isize, // 0 represents normal mode; x > 0 represents the server pid will switch to; -1 represents serving mode(forbid time interrupt)
}


impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            queue: VecDeque::new(),
            server: BTreeMap::new(),
            server_status: 0,
        }
    }

    pub fn set_server(&mut self, pid: isize) { self.server_status = pid; }

    pub fn is_fixed(&self) -> bool { self.server_status == -1 || self.server_status > 0 }

    pub fn add_server(&mut self, task: Arc<TaskControlBlock>) {
        self.server.insert(task.pid as isize, task);
    }

    pub fn add_task(&mut self, task: Arc<TaskControlBlock>) {
        if self.server_status == 0 {
            self.queue.push_back(task);
        } else if self.server_status > 0 {
            self.queue.push_front(task);
        }
    }

    pub fn fetch_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.server_status == 0 {
            return self.queue.pop_front();
        } else if self.server_status > 0 {
            let pid = self.server_status;
            self.server_status = -1;
            return self.server.get(&pid).cloned();
        } else if self.server_status == -1 {
            self.server_status = 0;
            return self.queue.pop_front();
        }
        return None;
    }

    pub fn remove_task(&mut self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        let mut to_remove: usize = 0xffffffff;
        for (index, task) in self.queue.iter().enumerate() {
            if task.pid == pid {
                to_remove = index;
                break;
            }
        }
        if to_remove != 0xffffffff {
            return self.queue.remove(to_remove);
        }
        return None;
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: SafeCellSingle<TaskManager> = unsafe {
        SafeCellSingle::new(TaskManager::new())
    };
}


pub fn set_server(pid: usize) {
    TASK_MANAGER.borrow_exclusive().set_server(pid as isize);
}

pub fn is_fixed() -> bool {
    TASK_MANAGER.borrow_exclusive().is_fixed()
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.borrow_exclusive().add_task(task);
}

pub fn add_server(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.borrow_exclusive().add_server(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.borrow_exclusive().fetch_task()
}

pub fn remove_task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.borrow_exclusive().remove_task(pid)
}