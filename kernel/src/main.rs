#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;
#[macro_use]
extern crate bitflags;


use core::arch::{asm, global_asm};

use riscv::register::{mepc, mstatus, pmpaddr0, pmpcfg0, satp, sie};

use crate::mm::init_mm;
use crate::syscall::syscall;

mod lang_items;
#[macro_use]
mod console;
mod sync;
mod sbi;
mod trap;
mod syscall;
mod mm;
mod task;
mod loader;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    init_mm();
    println!("Hello, world!");
    task::add_initproc();
    println!("after initproc!");
    loader::list_apps();
    task::run_tasks();
    task::suspend_current_and_run_next();
    panic!("Shutdown machine!");
}

// transfer state from M-mode to S-mode
#[no_mangle]
unsafe fn init() -> ! {
    mstatus::set_mpp(mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);
    satp::write(0);
    asm!(
    "csrw medeleg, {medeleg}",
    "csrw mideleg, {mideleg}",
    medeleg = in(reg) 0xffff,
    mideleg = in(reg) 0xffff,
    );
    sie::set_ssoft();
    sie::set_sext();
    sie::set_stimer();
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);
    //TODO:时钟中断
    asm!(
    "mret",
    options(noreturn),
    );
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}
