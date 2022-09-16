// SPDX-License-Identifier: MIT
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)] // FIXME

use crate::{__IncompleteArrayField, cmd, d, srv};
use libc::{c_int, c_uint, c_ulong, c_void};
use std::ptr;
use std::ptr::addr_of_mut;

/// ublksrv_aio_ctx is used to offload IO handling from ublksrv io_uring
/// context.
///
/// ublksrv_aio_ctx is bound with one single pthread which has to belong
/// to same process of the io_uring where IO is originated, so we can
/// support to handle IO from multiple queues of the same device. At
/// default, ublksrv_aio_ctx supports to handle device wide aio or io
/// offloading except for UBLKSRV_AIO_QUEUE_WIDE.
///
/// Meantime ublksrv_aio_ctx can be created per each queue, and only handle
/// IOs from this queue.
///
/// The final io handling in the aio context depends on user's implementation,
/// either sync or async IO submitting is supported.
pub const UBLKSRV_AIO_QUEUE_WIDE: u32 = 1 << 0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_aio_ctx {
    pub submit: ublksrv_aio_list,
    /// per-queue completion list
    pub complete: *mut ublksrv_aio_list,
    /// for wakeup us
    pub efd: c_int,
    pub flags: c_uint,
    pub dead: bool,
    pub dev: *mut srv::ublksrv_dev,
    pub ctx_data: *mut c_void,
}
d!(ublksrv_aio_ctx);

/// return value:
///
/// > 0 : the request is done
/// = 0 : submitted successfully, but not done
/// < 0 : submitted not successfully
pub type ublksrv_aio_submit_fn = Option<
    unsafe extern "C" fn(ctx: *mut ublksrv_aio_ctx, req: *mut ublksrv_aio) -> ::std::os::raw::c_int,
>;

pub unsafe fn ublksrv_aio_qid(val: c_uint) -> c_uint {
    (val >> 13) & 0x7ff
}

pub unsafe fn ublksrv_aio_tag(val: c_uint) -> c_uint {
    val & 0x1fff
}

pub unsafe fn ublksrv_aio_pid_tag(qid: c_uint, tag: c_uint) -> c_uint {
    tag | (qid << 13)
}

#[repr(C)]
pub struct ublksrv_aio {
    pub io: cmd::ublksrv_io_desc,
    pub union: ublksrv_aio_union_ty,
    /// reserved 31 ~ 24, bit 23 ~ 13: qid, bit 12 ~ 0: tag
    pub id: c_uint,
    pub next: *mut ublksrv_aio,
    pub data: __IncompleteArrayField<c_ulong>,
}
d!(ublksrv_aio);

#[repr(C)]
#[derive(Copy, Clone)]
pub union ublksrv_aio_union_ty {
    /// output
    pub res: c_int,
    /// input
    pub fd: c_int,
}
d!(ublksrv_aio_union_ty);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_aio_list {
    pub lock: libc::pthread_spinlock_t,
    pub list: aio_list,
}
d!(ublksrv_aio_list);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct aio_list {
    pub head: *mut ublksrv_aio,
    pub tail: *mut ublksrv_aio,
}
d!(aio_list);

pub unsafe fn aio_list_init(al: *mut aio_list) {
    (*al).head = ptr::null_mut();
    (*al).tail = ptr::null_mut();
}

pub unsafe fn aio_list_add(al: *mut aio_list, io: *mut ublksrv_aio) {
    (*io).next = ptr::null_mut();

    if !(*al).tail.is_null() {
        (*(*al).tail).next = io;
    } else {
        (*al).head = io;
    }

    (*al).tail = io;
}

pub unsafe fn aio_list_splice(n: *mut aio_list, head: *mut aio_list) {
    if (*n).head.is_null() {
        return;
    }

    if !(*head).tail.is_null() {
        (*(*head).tail).next = (*n).head;
    } else {
        (*head).head = (*n).head;
    }

    (*head).tail = (*n).tail;
    aio_list_init(n);
}

pub unsafe fn aio_list_empty(al: *const aio_list) -> bool {
    (*al).head.is_null()
}

pub unsafe fn aio_list_pop(al: *mut aio_list) -> *mut ublksrv_aio {
    let io = (*al).head;

    if !io.is_null() {
        (*al).head = (*io).next;
        if (*al).head.is_null() {
            (*al).tail = ptr::null_mut();
        }

        (*io).next = ptr::null_mut();
    }
    io
}

pub unsafe fn ublksrv_aio_ctx_dead(ctx: *const ublksrv_aio_ctx) -> bool {
    (*ctx).dead
}

pub unsafe fn ublksrv_aio_init_list(l: *mut ublksrv_aio_list) {
    libc::pthread_spin_init(addr_of_mut!((*l).lock), libc::PTHREAD_PROCESS_PRIVATE);
    aio_list_init(addr_of_mut!((*l).list));
}

extern "C" {
    pub fn ublksrv_aio_ctx_init(
        dev: *mut srv::ublksrv_dev,
        flags: ::std::os::raw::c_uint,
    ) -> *mut ublksrv_aio_ctx;
    pub fn ublksrv_aio_ctx_shutdown(ctx: *mut ublksrv_aio_ctx);
    pub fn ublksrv_aio_ctx_deinit(ctx: *mut ublksrv_aio_ctx);
    pub fn ublksrv_aio_alloc_req(
        ctx: *mut ublksrv_aio_ctx,
        payload_size: ::std::os::raw::c_int,
    ) -> *mut ublksrv_aio;
    pub fn ublksrv_aio_free_req(ctx: *mut ublksrv_aio_ctx, req: *mut ublksrv_aio);
    pub fn ublksrv_aio_submit_req(
        ctx: *mut ublksrv_aio_ctx,
        q: *mut srv::ublksrv_queue,
        req: *mut ublksrv_aio,
    );
    pub fn ublksrv_aio_get_completed_reqs(
        ctx: *mut ublksrv_aio_ctx,
        q: *const srv::ublksrv_queue,
        al: *mut aio_list,
    );
    pub fn ublksrv_aio_submit_worker(
        ctx: *mut ublksrv_aio_ctx,
        fn_: ublksrv_aio_submit_fn,
        submitted: *mut aio_list,
    ) -> ::std::os::raw::c_int;
    pub fn ublksrv_aio_complete_worker(ctx: *mut ublksrv_aio_ctx, completed: *mut aio_list);
    pub fn ublksrv_aio_handle_event(ctx: *mut ublksrv_aio_ctx, q: *mut srv::ublksrv_queue);
}
