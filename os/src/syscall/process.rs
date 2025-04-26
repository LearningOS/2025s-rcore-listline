//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,get_syscall_count,current_user_token};
use crate::mm::{VirtAddr, PageTable};


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
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    -1
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            // Read one byte from user address
            let ptr = id as *const u8;
            let token = current_user_token();
            let va = VirtAddr::from(ptr as usize);
            let page_table = PageTable::from_token(token);
            
            // Get the page table entry for the virtual address
            if let Some(pte) = page_table.translate(va.floor()) {
                // Check if the page is valid and readable
                if pte.is_valid() && pte.readable() {
                    let ppn = pte.ppn();
                    let offset = va.page_offset();
                    // Safe access to physical memory
                    let value = ppn.get_bytes_array()[offset];
                    return value as isize;
                }
            }
            // Address is not valid or not readable
            -1
        },
        1 => {
            // Write one byte to user address
            let ptr = id as *mut u8;
            let token = current_user_token();
            let va = VirtAddr::from(ptr as usize);
            let page_table = PageTable::from_token(token);
            
            // Get the page table entry for the virtual address
            if let Some(pte) = page_table.translate(va.floor()) {
                // Check if the page is valid and writable
                if pte.is_valid() && pte.writable() {
                    let ppn = pte.ppn();
                    let offset = va.page_offset();
                    // Safe write to physical memory
                    ppn.get_bytes_array()[offset] = data as u8;
                    return 0;
                }
            }
            // Address is not valid or not writable
            -1
        }
        2 => {
            // 获取系统调用计数 (包括本次调用)
            // 注意：本次系统调用已经在 syscall 函数中计数了
            get_syscall_count(id) as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
