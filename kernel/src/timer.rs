use core::arch::global_asm;

use riscv::register::{mhartid, mie, mscratch, mstatus, mtvec, time};

pub const TIME_INTERVAL: usize = 100000;
pub const CLOCK_FREQ: usize = 12500000;

global_asm!(include_str!("time_handler.S"));


#[link_section = ".bss.stack"]
#[no_mangle]
pub static mut SCRATCH: [usize; 5] = [0; 5];

pub unsafe fn init_timer() {
    let hart_id = mhartid::read();
    let mtimecmp = (0x02004000 + 8 * hart_id) as *mut usize;
    *mtimecmp = get_time() + TIME_INTERVAL;
    SCRATCH[3] = 0x02004000 + 8 * hart_id;
    SCRATCH[4] = TIME_INTERVAL;
    mscratch::write(SCRATCH.as_ptr() as usize);
    extern "C" {
        fn _time_handler();
    }
    mtvec::write(_time_handler as usize, mtvec::TrapMode::Direct);
    mstatus::set_mie();
    mie::set_mtimer();
}

pub fn get_time() -> usize {
    unsafe { (0x0200bff8 as *const usize).read_volatile() }
}

pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / 1000)
}