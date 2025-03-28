//! Process management syscalls
use core::mem::size_of;

use alloc::slice;

use crate::{
    mm::{translated_byte_buffer, PageTable, VirtAddr},
    task::{
        change_program_brk, current_map, current_munmap, current_user_token,
        exit_current_and_run_next, get_syscall, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

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

/// copy memory to user spcae
fn copy_to_user(kernel_start: usize, user_start: *const u8, _len: usize) {
    let mut copied_len = 0;
    let token = current_user_token();
    let slices = translated_byte_buffer(token, user_start, _len);
    for slice in slices {
        slice.clone_from_slice(unsafe {
            slice::from_raw_parts((kernel_start + copied_len) as *const u8, slice.len())
        });
        copied_len += slice.len();
    }
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    copy_to_user(
        &ts as *const TimeVal as usize,
        _ts as *const u8,
        size_of::<TimeVal>(),
    );
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    match _trace_request {
        // read
        0 => {
            let page_table = PageTable::from_token(token);
            let va = VirtAddr::from(_id);
            let vpn = va.floor();
            if let Some(pte) = page_table.translate(vpn) {
                if pte.is_valid() && pte.is_user() && pte.readable() {
                    return pte.ppn().get_bytes_array()[va.page_offset()].into();
                }
            }
        }
        // write()
        1 => {
            let page_table = PageTable::from_token(token);
            let va = VirtAddr::from(_id);
            let vpn = va.floor();
            if let Some(pte) = page_table.translate(vpn) {
                if pte.is_valid() && pte.is_user() && pte.writable() {
                    pte.ppn().get_bytes_array()[va.page_offset()] = _data as u8;
                    return 0;
                }
            }
        }
        // syscall
        2 => return get_syscall(_id) as isize,
        // default
        _ => (),
    }
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if current_map(start, len, port) {
        0
    } else {
        -1
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    trace!("kernel: sys_munmap");
    if current_munmap(start, len) {
        0
    } else {
        -1
    }
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
