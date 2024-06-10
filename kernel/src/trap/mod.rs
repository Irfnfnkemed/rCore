use core::arch::{asm, global_asm};

use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

use crate::mm::memory_set::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{current_trap_cx, current_user_token, exit_current_and_run_next};
use crate::trap::context::TrapContext;

pub(crate) mod context;

global_asm!(include_str!("trap.S"));


#[no_mangle]
pub fn trap_handler() -> ! {
   // println!("[trap] Begin to trap in.");
    let scause = scause::read();
    let stval = stval::read();

    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct); // if trap in kernel
    }
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let cx = current_trap_cx();
            cx.sepc += 4;
            let return_var = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12],
                cx.x[13], cx.x[14], cx.x[15], cx.x[16]]) as usize;
            let cx = current_trap_cx(); // trap_cx may change after sys_call
            cx.x[10] = return_var;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
   // println!("[trap] Begin to trap out.");
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
    panic!("[trap] Unreachable in back_to_user!");
}

#[no_mangle]
pub fn trap_from_kernel() -> ! {
    panic!("[trap] A trap from kernel!");
}
