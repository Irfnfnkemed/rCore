#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::string::String;
use alloc::vec::Vec;

use user_lib::{exec, fork, kill, read, wait};

const STDIN: usize = 0;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const ETX: u8 = 0x03u8;
const BS: u8 = 0x08u8;
const DEL: u8 = 0x7fu8;

#[no_mangle]
fn main() -> i32 {
    let mut buf = [0u8; 1];
    let mut cmd: String = String::new();
    print!("\x1b[31m[kill] kill which?.\n>> pid = \x1b[0m");
    loop {
        read(STDIN, buf.as_mut());
        match buf[0] {
            LF | CR => {
                print!("\n");
                let pid: usize = cmd.parse().unwrap();
                kill(pid as isize, 9);
                cmd.clear();
                print!("\x1b[31m[kill] kill which?.\n>> pid = \x1b[0m");
            }
            BS | DEL => {
                if !cmd.is_empty() {
                    print!("{}", BS as char); // control the cursor
                    print!(" "); // cover the old char
                    print!("{}", BS as char); // control the cursor again
                    cmd.pop();
                }
            }
            _ => {
                print!("{}", buf[0] as char);
                cmd.push(buf[0] as char);
            }
        }
    }
}