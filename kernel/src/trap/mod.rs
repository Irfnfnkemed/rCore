use core::arch::{asm, global_asm};

use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

use crate::mm::memory_set::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{current_trap_cx, current_user_token};
use crate::trap::context::TrapContext;

pub(crate) mod context;

global_asm!(include_str!("trap.S"));


#[no_mangle]
pub fn trap_handler() -> ! {
    let scause = scause::read();
    let stval = stval::read();
    let cx = current_trap_cx();
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct); // if trap in kernel
    }
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12],
                cx.x[13], cx.x[14], cx.x[15], cx.x[16]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
        "fence.i",
        "jr {restore_va}",
        restore_va = in(reg) restore_va,
        in("a0") trap_cx_ptr,
        in("a1") user_satp,
        options(noreturn)
        );
    }
    panic!("Unreachable in back_to_user!");
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!");
}
