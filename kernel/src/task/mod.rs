use alloc::sync::{Arc, Weak};

use lazy_static::lazy_static;

pub use manager::add_task;
pub use processor::{current_task, current_trap_cx, current_user_token};

use crate::loader::get_app_data_by_name;
use crate::task::context::TaskContext;
use crate::task::processor::{schedule, take_current_task};
use crate::task::task::{TaskControlBlock, TaskStatus};

mod pid;
mod stack;
mod task;
mod manager;
mod processor;
mod switch;
mod context;

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(
        TaskControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}


pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap(); // move curr-task
    let mut task_inner = task.borrow_exclusive_inner();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;// Change status to Ready
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap(); // move curr-task
    let mut task_inner = task.borrow_exclusive_inner();
    task_inner.task_status = TaskStatus::Zombie;
    task_inner.exit_code = exit_code;
    // move all the child-proc to INIT_PROC
    let mut initproc_inner = INITPROC.borrow_exclusive_inner();
    for child in task_inner.children.iter() {
        child.borrow_exclusive_inner().parent = Some(Arc::downgrade(&INITPROC));
        initproc_inner.children.push(child.clone());
    }
    drop(initproc_inner);
    task_inner.children.clear();
    task_inner.memory_set.recycle();
    drop(task_inner);
    drop(task);
    let mut _unused = TaskContext::new_zero();
    schedule(&mut _unused as *mut _);
}