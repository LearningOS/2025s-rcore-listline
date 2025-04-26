//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,get_syscall_count,current_user_token};
use crate::mm::{translated_byte_buffer, VirtAddr, PageTable, MemorySet};
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
            -1
        }
        2 => {
            // 获取系统调用计数 (包括本次调用)
            get_syscall_count(id) as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    
    let mut memory_set = MemorySet::new_bare();
    memory_set.mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    
    let mut memory_set = MemorySet::new_bare();
    memory_set.unmmap(start, len)
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