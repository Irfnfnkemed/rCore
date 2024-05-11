use core::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicPtr, Ordering};
mod uart;

const SHUT_DOWN_ADDR: usize = 0x100000;
const SHUT_DOWN_FLAG: u32 = 0x5555;

pub fn print(args: fmt::Arguments) {
    uart::UART.borrow_exclusive().write_fmt(args).unwrap();
}

pub fn shutdown() -> ! {
    let tmp = AtomicPtr::new(SHUT_DOWN_ADDR as *mut u32).load(Ordering::Acquire);
    unsafe { tmp.write(SHUT_DOWN_FLAG); }
    unreachable!()
}