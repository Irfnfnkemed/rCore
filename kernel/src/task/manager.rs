use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use lazy_static::lazy_static;

use crate::sync::safe_cell_single::SafeCellSingle;
use crate::task::rand::LinearCongruentialGenerator;
use crate::task::task::TaskControlBlock;

const DEFAULT_LOTTERY_SHARE: usize = 100;
const DEFAULT_PRIORITY: i32 = 10;
const PRIORITY_LOWER_BOUND: usize = 5;
const PRIORITY_SHARE: usize = 5;

pub struct Lottery {
    pid: usize,
    share: usize,
    priority: i32,
}

impl Lottery {
    pub fn new(pid: usize) -> Self {
        Lottery {
            pid: pid,
            share: DEFAULT_LOTTERY_SHARE,
            priority: DEFAULT_PRIORITY,
        }
    }

    pub fn reduce(&mut self) {
        self.share -= 1;
        if self.share < self.priority as usize * PRIORITY_LOWER_BOUND {
            self.priority -= 1;
            self.share += self.priority as usize * PRIORITY_SHARE;
            if self.priority == 0 {
                self.priority = DEFAULT_PRIORITY;
                self.share = DEFAULT_LOTTERY_SHARE;
            }
        }
    }
}

pub struct TaskManager {
    user: BTreeMap<usize, Arc<TaskControlBlock>>,
    server: BTreeMap<isize, Arc<TaskControlBlock>>,
    lottery: Vec<Lottery>,
    wait: Option<Arc<TaskControlBlock>>,
    rand: LinearCongruentialGenerator,
    sum_lottery: usize,
    server_status: isize, // 0 represents normal mode; x > 0 represents the server pid will switch to; -1 represents serving mode(forbid time interrupt)
}


impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            user: BTreeMap::new(),
            server: BTreeMap::new(),
            lottery: Vec::new(),
            wait: None,
            rand: LinearCongruentialGenerator::new(1664525, 1013904223, 2usize.pow(32), 123456789),
            sum_lottery: 0,
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
            if self.lottery.iter().find(|&x| x.pid == task.pid).is_none() {
                self.lottery.push(Lottery::new(task.pid));
                self.sum_lottery += DEFAULT_LOTTERY_SHARE;
            }
            self.user.insert(task.pid, task);
        } else if self.server_status > 0 {
            self.wait = Some(task);
        }
    }

    pub fn fetch_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut sum = 0;
        for elem in self.lottery.iter_mut() {
            sum += elem.share;
        }
        if self.server_status == 0 {
            let id: usize = self.rand.next() % self.sum_lottery + 1;
            let mut sum: usize = 0;
            let mut pid: usize = 0xffffffff;
            for elem in self.lottery.iter_mut() {
                sum += elem.share;
                if sum >= id {
                    pid = elem.pid;
                    self.sum_lottery -= elem.share;
                    elem.reduce();
                    self.sum_lottery += elem.share;
                    break;
                }
            }
            return self.user.remove(&pid);
        } else if self.server_status > 0 {
            let pid = self.server_status;
            self.server_status = -1;
            return self.server.get(&pid).cloned();
        } else if self.server_status == -1 {
            self.server_status = 0;
            return self.wait.take();
        }
        return None;
    }


    pub fn remove_task(&mut self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        let mut to_remove: usize = 0xffffffff;
        for (index, task) in self.lottery.iter().enumerate() {
            if task.pid == pid {
                to_remove = index;
                break;
            }
        }
        if to_remove != 0xffffffff {
            self.lottery.remove(to_remove);
        }
        return self.user.remove(&pid);
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