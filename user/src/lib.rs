#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use crate::buddy::{Allocator, AllocatorWrap};
use crate::syscall::{sys_exec, sys_exit, sys_fork, sys_waitpid, sys_write};

mod buddy;
mod syscall;
mod lang_items;
mod console;
pub mod sbi;
mod sync;

const USER_HEAP_SIZE: usize = 16384;

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

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}

pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                // TODO: yield
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                // TODO: yield
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
