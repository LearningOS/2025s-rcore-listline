//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,get_syscall_count,
    current_user_token,make_mmap,make_munmap};
use crate::mm::{translated_byte_buffer, VirtAddr, PageTable,PhysAddr};
use crate::timer::get_time_us;


#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [TimeVal] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let token = current_user_token();
    let len = core::mem::size_of::<TimeVal>();
    let buffers = translated_byte_buffer(token, ts as *const u8, len);

    if buffers.iter().map(|b| b.len()).sum::<usize>() != len {
        return -1;
    }

    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    // 把结构体分块拷贝到用户空间
    let src = unsafe {
        core::slice::from_raw_parts(
            &time_val as *const TimeVal as *const u8,
            len,
        )
    };

    let mut offset = 0;
    for dst in buffers {
        let end = offset + dst.len();
        dst.copy_from_slice(&src[offset..end]);
        offset = end;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let page_table = PageTable::from_token(current_user_token());
    let virtaddr = VirtAddr::from(id);
    let pte = match page_table.translate(virtaddr.floor()) {
        Some(pte) => pte,
        None => return -1, 
    };
    match trace_request {
        0 => {
            if pte.is_user() && pte.readable() {
                let physaddr: PhysAddr = pte.ppn().into();
                let addr = physaddr.0 | virtaddr.page_offset();
                let raw_ptr = addr as *const u8;
                unsafe { *raw_ptr as isize }
            } else {
                -1
            }
        },
        1 => {
            if pte.is_user() && pte.writable() {
                let physaddr: PhysAddr = pte.ppn().into();
                let addr = physaddr.0 | virtaddr.page_offset();
                let raw_ptr = addr as *mut u8;
                unsafe {
                    *raw_ptr = data as u8;
                    0
                }
            } else {
                -1
            }
        },
        2 => {
            // 获取系统调用计数 (包括本次调用)
            get_syscall_count(id) as isize
        },
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    
    make_mmap(start, len, port)

}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    
    make_munmap(start, len)

}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}