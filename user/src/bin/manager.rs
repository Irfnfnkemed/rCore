#![no_std]
#![no_main]
extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

use lazy_static::lazy_static;

use user_lib::sync::safe_cell_single::SafeCellSingle;
use user_lib::yield_;

const BUFFER: usize = usize::MAX - 0x3000 + 1;
const PAGE_SIZE: usize = 0x1000;

const NOP_REQUEST: usize = 0;
const FORK_REQUEST: usize = 1;
const EXIT_REQUEST: usize = 2;
const WAITPID_REQUEST: usize = 3;
const DONE_REQUEST: usize = 4;

pub struct PidHandle(pub usize);

pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.borrow_exclusive().dealloc(self.0);
    }
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        } // pid=0: initproc; pid=1: manager
    }

    pub fn alloc(&mut self) -> PidHandle {
        let mut pid: usize = 0;
        if self.recycled.is_empty() {
            pid = self.current;
            self.current += 1;
        } else {
            pid = self.recycled.pop().unwrap();
        }
        PidHandle(pid)
    }

    pub fn dealloc(&mut self, pid: usize) {
        self.recycled.push(pid);
    }
}


pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.borrow_exclusive().alloc()
}

pub fn pid_dealloc(pid: usize) {
    PID_ALLOCATOR.borrow_exclusive().dealloc(pid);
}

pub struct ProcessControlBlock {
    pub pid: PidHandle,
    inner: SafeCellSingle<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
    pub is_zombie: bool,
}

impl ProcessControlBlock {
    pub fn new() -> Self {
        Self {
            pid: pid_alloc(),
            inner: unsafe {
                SafeCellSingle::new(
                    ProcessControlBlockInner {
                        parent: None,
                        children: Vec::new(),
                        exit_code: 0,
                        is_zombie: false,
                    }
                )
            },
        }
    }

    pub fn borrow_exclusive_inner(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.borrow_exclusive()
    }
}


lazy_static! {
    static ref PID_ALLOCATOR : SafeCellSingle<PidAllocator> = unsafe {
        SafeCellSingle::new(PidAllocator::new())
    };
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(
        ProcessControlBlock::new()
    );
}

#[no_mangle]
fn main() -> i32 {
    let mut processes: BTreeMap<usize, Arc<ProcessControlBlock>> = BTreeMap::new();
    processes.insert(0, INITPROC.clone());
    let manager = Arc::new(ProcessControlBlock::new());
    INITPROC.borrow_exclusive_inner().children.push(manager.clone());
    manager.borrow_exclusive_inner().parent = Some(Arc::downgrade(&INITPROC.clone()));
    let buffer_usize = unsafe { core::slice::from_raw_parts_mut(BUFFER as *mut usize, PAGE_SIZE / 8) };
    loop {
        {
            if buffer_usize[0] == FORK_REQUEST {
                let cur_pid = buffer_usize[1];
                let cur_proc = processes.get(&cur_pid).unwrap().clone();
                let new_proc = Arc::new(ProcessControlBlock::new());
                new_proc.borrow_exclusive_inner().parent = Some(Arc::downgrade(&cur_proc.clone()));
                cur_proc.borrow_exclusive_inner().children.push(new_proc.clone());
                processes.insert(new_proc.pid.0, new_proc.clone());
                buffer_usize[0] = DONE_REQUEST;
                buffer_usize[1] = new_proc.pid.0;
                continue;
            } else if buffer_usize[0] == EXIT_REQUEST {
                let cur_pid = buffer_usize[1];
                let exit_code = buffer_usize[2];
                let cur_proc = processes.get(&cur_pid).unwrap().clone();
                let mut cur_inner = cur_proc.borrow_exclusive_inner();
                cur_inner.exit_code = exit_code as i32;
                cur_inner.is_zombie = true;
                let mut initproc_inner = INITPROC.borrow_exclusive_inner();
                for child in cur_inner.children.iter() {
                    child.borrow_exclusive_inner().parent = Some(Arc::downgrade(&INITPROC));
                    initproc_inner.children.push(child.clone());
                }
                cur_inner.children.clear();
                buffer_usize[0] = DONE_REQUEST;
                continue;
            } else if buffer_usize[0] == WAITPID_REQUEST {
                let cur_pid = buffer_usize[1];
                let wait_pid = buffer_usize[2] as isize;
                let cur_proc = processes.get(&cur_pid).unwrap().clone();
                let mut cur_inner = cur_proc.borrow_exclusive_inner();
                if !cur_inner.children.iter().
                    any(|son| { wait_pid == -1 || son.pid.0 == wait_pid as usize }) {
                    buffer_usize[0] = DONE_REQUEST;
                    buffer_usize[1] = (-1i32) as usize;
                    continue;
                }
                let pair = cur_inner.children.iter().enumerate().
                    find(|(_, son)| {
                        (wait_pid == -1 || son.pid.0 == wait_pid as usize) && son.borrow_exclusive_inner().is_zombie
                    });
                if let Some((index, _)) = pair {
                    let mut child = cur_inner.children.remove(index);
                    processes.remove(&child.pid.0);
                    assert_eq!(Arc::strong_count(&child), 1); // confirm, child-proc should be only owned here
                    buffer_usize[0] = DONE_REQUEST;
                    buffer_usize[1] = child.pid.0;
                    buffer_usize[2] = child.borrow_exclusive_inner().exit_code as usize;
                    continue;
                } else {
                    buffer_usize[0] = DONE_REQUEST;
                    buffer_usize[1] = (-2i32) as usize;
                    continue;
                }
            } else if buffer_usize[0] != NOP_REQUEST && buffer_usize[0] != DONE_REQUEST {
                println!("[Manager] Unknown request!");
            }
        } // drop inner_borrow
        yield_();
    }
    0
}
