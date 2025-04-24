//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next,get_syscall_count},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    
    match trace_request {
        0 => {
            // 读取用户地址处的一个字节
            let ptr = id as *const u8;
            // 安全：根据需求，我们不做安全检查
            unsafe { *ptr as isize }
        }
        1 => {
            // 写入一个字节到用户地址
            let ptr = id as *mut u8;
            // 安全：根据需求，我们不做安全检查
            unsafe { 
                *ptr = data as u8;
                0 // 返回0
            }
        }
        2 => {
            // 获取系统调用计数 (包括本次调用)
            // 注意：本次系统调用已经在 syscall 函数中计数了
            get_syscall_count(id) as isize
        }
        _ => -1,
    }
}
