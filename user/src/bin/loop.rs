#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let mut i: usize = 0;
    loop {
        if i % 1000000000 == 0 {
            println!("looping...");
        }
        i += 1;
    }
    0
}