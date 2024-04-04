#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::{asm, global_asm};

use riscv::register::{mepc, mstatus, pmpaddr0, pmpcfg0, satp, sie};

use crate::syscall::syscall;

mod lang_items;
#[macro_use]
mod console;
mod sync;
mod sbi;
mod trap;
mod bach;
mod syscall;
mod mm;

global_asm!(include_str!("entry.asm"));


#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    let array: [u8; 10] = [u8::try_from('a').unwrap(); 10];
    syscall(64, [1, &array[0] as *const u8 as usize, 10]);
    syscall(93, [4, 0, 0]);
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