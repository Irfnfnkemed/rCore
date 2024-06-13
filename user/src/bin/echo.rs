#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::string::String;
use alloc::vec::Vec;

use user_lib::{exec, fork, read};

const STDIN: usize = 0;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const ETX: u8 = 0x03u8;
const BS: u8 = 0x08u8;
const DEL: u8 = 0x7fu8;

#[no_mangle]
fn main() -> i32 {
    let mut buf = [0u8; 1];
    let mut line = String::new();
    loop {
        read(STDIN, &mut buf);
        match buf[0] {
            LF | CR => {
                print!("\n");
                println!("{}", line);
                break;
            }
            BS | DEL => {
                if !line.is_empty() {
                    print!("{}", BS as char); // control the cursor
                    print!(" "); // cover the old char
                    print!("{}", BS as char); // control the cursor again
                    line.pop();
                }
            }
            _ => {
                print!("{}", buf[0] as char);
                line.push(buf[0] as char);
            }
        }
    }
    0
}