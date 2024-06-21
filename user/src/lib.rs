#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use crate::buddy::{Allocator, AllocatorWrap};
use crate::syscall::{sys_exec, sys_exit, sys_fork, sys_get_time, sys_getpid, sys_kill, sys_read, sys_waitpid, sys_write, sys_yield};

mod buddy;
mod syscall;
mod lang_items;
pub mod console;
pub mod sync;

const USER_HEAP_SIZE: usize = 0x4000;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

static mut INNER_ALLOCATOR: Allocator = Allocator::empty();

#[global_allocator]
static mut HEAP_ALLOCATOR: AllocatorWrap = AllocatorWrap::empty();

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // don't need to clear .bss, since it's done when loading ELF
    init_heap();
    exit(main());
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!"); // if don't find main function in app
}

pub fn init_heap() {
    unsafe {
        INNER_ALLOCATOR.init(HEAP_SPACE.as_ptr() as usize);
        HEAP_ALLOCATOR.allocator = &mut INNER_ALLOCATOR as *mut Allocator as usize;
    }
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf, buf.len())
}

pub fn read_without_block(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf, 0)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn wait(pid: isize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    sys_waitpid(pid as isize, exit_code as *mut _)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn kill(pid: isize, signal: u8) -> isize {
    sys_kill(pid, signal)
}

pub fn get_time() -> isize {
    sys_get_time()
}
pub fn getpid() -> isize {
    sys_getpid()
}

pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}