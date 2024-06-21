#![no_std]
#![no_main]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::string::String;
use alloc::vec::Vec;

use user_lib::{exec, fork, kill, read, read_without_block, wait, waitpid, yield_};

const STDIN: usize = 0;
const SIGKILL: u8 = 9;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const BS: u8 = 0x08u8;
const DEL: u8 = 0x7fu8;
const ETX: u8 = 0x03u8;

#[no_mangle]
fn main() -> i32 {
    println!("\x1b[32m[shell] Begin user shell.\n>> \x1b[0m");
    let mut buf = [0u8; 1];
    let mut cmd: String = String::new();
    loop {
        read(STDIN, buf.as_mut());
        match buf[0] {
            LF | CR => {
                print!("\n");
                if !cmd.is_empty() {
                    cmd.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        if exec(cmd.as_str()) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let mut buf = [0u8; 1];
                        let mut cnt: usize = 0;
                        loop {
                            cnt += 1;
                            match waitpid(pid as usize, &mut exit_code) {
                                -2 => {
                                    yield_();
                                }
                                exit_pid => { // -1 or a real pid
                                    assert_eq!(pid, exit_pid);
                                    println!("[shell] Process {} exited with code {}", pid, exit_code);
                                    break;
                                }
                            }
                            if cnt % 10 == 0 {
                                read_without_block(STDIN, buf.as_mut());
                                if buf[0] == ETX {
                                    if kill(pid, SIGKILL) == 0 {
                                        let kill_pid = waitpid(pid as usize, &mut exit_code);
                                        assert_eq!(pid, kill_pid);
                                        assert_eq!(exit_code, SIGKILL as i32);
                                        println!("[shell] Process {} is killed successfully.", pid);
                                    } else {
                                        println!("[shell] Process {} cannot be killed.", pid);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    cmd.clear();
                }
                print!("\x1b[32m>> \x1b[0m");
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