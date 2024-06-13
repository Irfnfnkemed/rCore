use alloc::string::String;
use core::slice::SliceIndex;

use crate::loader::get_app_data_by_name;
use crate::mm::address::VirtAddr;
use crate::mm::page_table::{PageTable, translated_byte_buffer};
use crate::task::{add_task, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::task::manager::remove_task;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const SIGKILL: u8 = 9;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            return if len == 0 {
                let mut buffers = translated_byte_buffer(current_user_token(), buf, 1);
                let ch = crate::sbi::recv();
                if ch == 0 {
                    0
                } else {
                    buffers[0][0] = ch;
                    1
                }
            } else {
                let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
                let mut ch: u8 = 0;
                let mut index_1 = 0;
                let mut index_2 = 0;
                let mut cnt = 0;
                while cnt < len {
                    ch = crate::sbi::recv();
                    if ch == 0 {
                        suspend_current_and_run_next();
                        continue;
                    } else {
                        buffers[index_1][index_2] = ch;
                        index_2 += 1;
                        if index_2 >= buffers[index_1].len() {
                            index_1 += 1;
                            index_2 = 0;
                        }
                        cnt += 1;
                    }
                }
                len as isize
            }
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}


pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}, pid = {}", exit_code, current_task().unwrap().pid);
    exit_current_and_run_next(exit_code);
    panic!("[kernel] Unreachable area in sys_exit!")
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid;
    println!("[kernel] Application forked (parent pid = {}, child pid = {})", current_task.pid, new_pid);
    let trap_cx = new_task.borrow_exclusive_inner().get_trap_cx();
    trap_cx.x[10] = 0;  // a0 =0
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let cur_task = current_task().unwrap();
    let token = cur_task.borrow_exclusive_inner().get_user_token();
    let tmp_page_table = PageTable::new_tmp(token);
    let mut path_str = String::new();
    let mut va = path as usize;
    loop {
        let pa = tmp_page_table.translate_va(VirtAddr::from(va)).unwrap();
        let ch: u8 = *(pa.get_mut());
        if ch == 0 {
            break;
        } else {
            path_str.push(ch as char);
            va += 1;
        }
    }
    println!("[kernel] Application executed (pid = {}, path = {})", cur_task.pid, path_str.as_str());
    if let Some(data) = get_app_data_by_name(path_str.as_str()) {
        cur_task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let cur_task = current_task().unwrap();
    cur_task.waitpid(pid, exit_code_ptr)
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_kill(pid: usize, signal: u8) -> isize {
    return match signal {
        SIGKILL => {
            if pid == 0 {
                println!("[kernel] Initproc cannot be killed!");
                -1
            } else if pid == 1 {
                println!("[kernel] Manager cannot be killed!");
                -1
            } else {
                let cur_pid = current_task().unwrap().pid;
                if cur_pid == pid {
                    println!("[kernel] Application (pid = {}) is killed by pid = {}.", pid, cur_pid);
                    exit_current_and_run_next(SIGKILL as i32);
                } else {
                    if let Some(kill_task) = remove_task(pid) {
                        kill_task.exit(SIGKILL as i32);
                    } else {
                        println!("[kernel] No application with pid = {}!", pid);
                        return -1;
                    }
                }
                println!("[kernel] Application (pid = {}) is killed by pid = {}.", pid, cur_pid);
                0
            }
        }
        _ => {
            println!("[kernel] Unsupported signal!");
            -1
        }
    };
    0
}