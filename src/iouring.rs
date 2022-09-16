// SPDX-License-Identifier: MIT
#![allow(non_camel_case_types)]
use crate::{__IncompleteArrayField, d};
use libc::{__s32, __u16, __u32, __u64, __u8, c_int, c_uint, c_void, size_t};

/// Minimal liburing 2.2 bindings
type __kernel_rwf_t = c_int; // linux/fs.h

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct io_uring {
    pub sq: io_uring_sq,
    pub cq: io_uring_cq,
    pub flags: c_uint,
    pub ring_fd: c_int,
    pub features: c_uint,
    pub enter_ring_fd: c_int,
    pub int_flags: __u8,
    pub pad: [__u8; 3],
    pub pad2: c_uint,
}
d!(io_uring);

#[repr(C)]
#[derive(Debug)]
pub struct io_uring_cqe {
    pub user_data: __u64,
    pub res: __s32,
    pub flags: __u32,
    pub big_cqe: __IncompleteArrayField<__u64>,
}
d!(io_uring_cqe);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct io_uring_sq {
    pub khead: *mut c_uint,
    pub ktail: *mut c_uint,
    pub kring_mask: *mut c_uint,
    pub kring_entries: *mut c_uint,
    pub kflags: *mut c_uint,
    pub kdropped: *mut c_uint,
    pub array: *mut c_uint,
    pub sqes: *mut io_uring_sqe,
    pub sqe_head: c_uint,
    pub sqe_tail: c_uint,
    pub ring_sz: size_t,
    pub ring_ptr: *mut c_void,
    pub pad: [c_uint; 4],
}
d!(io_uring_sq);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct io_uring_cq {
    pub khead: *mut c_uint,
    pub ktail: *mut c_uint,
    pub kring_mask: *mut c_uint,
    pub kring_entries: *mut c_uint,
    pub kflags: *mut c_uint,
    pub koverflow: *mut c_uint,
    pub cqes: *mut io_uring_cqe,
    pub ring_sz: size_t,
    pub ring_ptr: *mut c_void,
    pub pad: [c_uint; 4],
}
d!(io_uring_cq);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct io_uring_sqe {
    pub opcode: __u8,
    pub flags: __u8,
    pub ioprio: __u16,
    pub fd: __s32,
    pub u1: io_uring_sqe_union_1_ty,
    pub u2: io_uring_sqe_union_2_ty,
    pub len: __u32,
    pub u3: io_uring_sqe_union_3_ty,
    pub user_data: __u64,
    pub u4: io_uring_sqe_union_4_ty,
    pub personality: __u16,
    pub u5: io_uring_sqe_union_5_ty,
    pub addr3: __u64,
    pub __pad2: [__u64; 1usize],
}
d!(io_uring_sqe);

#[repr(C)]
#[derive(Copy, Clone)]
pub union io_uring_sqe_union_1_ty {
    pub off: __u64,
    pub addr2: __u64,
}
d!(io_uring_sqe_union_1_ty);

#[repr(C)]
#[derive(Copy, Clone)]
pub union io_uring_sqe_union_2_ty {
    pub addr: __u64,
    pub splice_off_in: __u64,
}
d!(io_uring_sqe_union_2_ty);

#[repr(C)]
#[derive(Copy, Clone)]
pub union io_uring_sqe_union_3_ty {
    pub rw_flags: __kernel_rwf_t,
    pub fsync_flags: __u32,
    pub poll_events: __u16,
    pub poll32_events: __u32,
    pub sync_range_flags: __u32,
    pub msg_flags: __u32,
    pub timeout_flags: __u32,
    pub accept_flags: __u32,
    pub cancel_flags: __u32,
    pub open_flags: __u32,
    pub statx_flags: __u32,
    pub fadvise_advice: __u32,
    pub splice_flags: __u32,
    pub rename_flags: __u32,
    pub unlink_flags: __u32,
    pub hardlink_flags: __u32,
    pub xattr_flags: __u32,
}
d!(io_uring_sqe_union_3_ty);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union io_uring_sqe_union_4_ty {
    pub buf_index: __u16,
    pub buf_group: __u16,
}
d!(io_uring_sqe_union_4_ty);

#[repr(C)]
#[derive(Copy, Clone)]
pub union io_uring_sqe_union_5_ty {
    pub splice_fd_in: __s32,
    pub file_index: __u32,
}
d!(io_uring_sqe_union_5_ty);
