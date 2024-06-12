#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;

use user_lib::{exec, fork, wait};

#[no_mangle]
fn main() -> i32 {
    let mut i: usize = 0;
    loop {
        if i % 10000000 == 0 {
            println!("looping...");
        }
        i += 1;
    }
    4
}