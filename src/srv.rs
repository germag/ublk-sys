// SPDX-License-Identifier: MIT
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)] // FIXME

use crate::iouring;
use crate::{__IncompleteArrayField, cmd, d};
use libc::{__u64, c_char, c_int, c_long, c_uint, c_ulong, c_ulonglong, c_ushort, c_void};

pub const MAX_NR_HW_QUEUES: u32 = 32;
pub const MAX_QD: u32 = 1024;
pub const MAX_BUF_SIZE: u32 = 1024 << 10;

pub const DEF_NR_HW_QUEUES: u32 = 1;
pub const DEF_QD: u32 = 256;
pub const DEF_BUF_SIZE: u32 = 512 << 10;

pub const UBLKSRV_SHM_DIR: &[u8; 8] = b"ublksrv\0";
pub const UBLKSRV_SHM_SIZE: u32 = 1024;

/// stored in ublksrv_ctrl_dev_info->ublksrv_flags

/// HAS_IO_DAEMON means io handler has its own daemon context which isn't
/// same with control command context, so shared memory communication is
/// required between control task and io daemon
pub const UBLKSRV_F_HAS_IO_DAEMON: u32 = 1;

/// target may not use io_uring for handling io, so eventfd is required
/// for wakeup io command io_uring context
pub const UBLKSRV_F_NEED_EVENTFD: u32 = 2;

// A opaque type
// TODO: Replace with a extern type once RFC 1861 becomes stable
// #![feature(extern_types)]
// extern "C" {
//     type ublksrv_aio_ctx;
// }
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_aio_ctx {
    _private: [u8; 0],
}

/// Generic data for creating one ublk device
///
/// Target specific data is handled by ->init_tgt
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_dev_data {
    pub dev_id: c_int,
    pub max_io_buf_bytes: c_uint,
    pub nr_hw_queues: c_ushort,
    pub queue_depth: c_ushort,
    pub tgt_type: *const c_char,
    pub tgt_ops: *const ublksrv_tgt_type,
    pub tgt_argc: c_int,
    pub tgt_argv: *mut *mut c_char,
    pub run_dir: *const c_char,
    pub flags: c_ulong,
    pub ublksrv_flags: c_ulong,
    pub reserved: [c_ulong; 7],
}
d!(ublksrv_dev_data);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_ctrl_dev {
    pub ring: iouring::io_uring,
    pub ctrl_fd: c_int,
    pub bs_shift: c_uint,
    pub dev_info: cmd::ublksrv_ctrl_dev_info,
    pub tgt_type: *const c_char,
    pub tgt_ops: *const ublksrv_tgt_type,
    /// default is UBLKSRV_RUN_DIR but can be specified via command line,
    /// pid file will be saved there
    pub run_dir: *const c_char,
    pub tgt_argc: c_int,
    pub tgt_argv: *mut *mut c_char,
    pub queues_cpuset: *mut libc::cpu_set_t,
}
d!(ublksrv_ctrl_dev);

pub const UBLKSRV_NEED_FETCH_RQ: u32 = 1 << 0;
pub const UBLKSRV_NEED_COMMIT_RQ_COMP: u32 = 1 << 1;
pub const UBLKSRV_IO_FREE: u32 = 1 << 2;
pub const UBLKSRV_NEED_GET_DATA: u32 = 1 << 3;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ublk_io {
    pub buf_addr: *mut c_char,
    pub flags: c_uint,
    pub union: ublk_io_anon_union_ty,
    pub tgt_io_cqe: *mut iouring::io_uring_cqe,
    pub io_data: c_ulong,
}
d!(ublk_io);

#[repr(C)]
#[derive(Copy, Clone)]
pub union ublk_io_anon_union_ty {
    /// result is updated after all target ios are done
    pub result: c_uint,
    /// current completed target io cqe
    pub queued_tgt_io: c_int,
}
d!(ublk_io_anon_union_ty);

pub const UBLKSRV_QUEUE_STOPPING: u32 = 1;
pub const UBLKSRV_QUEUE_IDLE: u32 = 2;
pub const UBLKSRV_NR_CTX_BATCH: u32 = 4;

#[repr(C)]
pub struct ublksrv_queue {
    pub q_id: c_int,
    pub q_depth: c_int,
    pub private_data: *mut c_void,
    /// Read only by ublksrv daemon, setup via mmap on /dev/ublkcN.
    ///
    /// ublksrv_io_desc(iod) is stored in this buffer, so iod
    /// can be retrieved by request's tag directly.
    ///
    /// ublksrv writes the iod into this array, and notify ublksrv daemon
    /// by issued io_uring command beforehand.
    pub io_cmd_buf: *mut c_char,
    pub io_buf: *mut c_char,
    pub cmd_inflight: c_uint,
    pub tgt_io_inflight: c_uint,
    pub state: c_uint,
    /// eventfd
    pub efd: c_int,
    /// cache tgt ops
    pub tgt_ops: *const ublksrv_tgt_type,
    /// ring for submit io command to ublk driver, can only be issued from ublksrv daemon.
    /// ring depth == dev_info->queue_depth.
    pub ring: iouring::io_uring,
    pub dev: *mut ublksrv_dev,
    pub tid: c_uint,
    pub nr_ctxs: c_int,
    pub ctxs: [*mut ublksrv_aio_ctx; UBLKSRV_NR_CTX_BATCH as usize],
    pub ios: __IncompleteArrayField<ublk_io>,
}
d!(ublksrv_queue);

pub const UBLKSRV_TGT_MAX_FDS: u32 = 32;

// enum
/// evaluate communication cost, ublksrv_null vs /dev/nullb0
pub const UBLKSRV_TGT_TYPE_NULL: c_uint = 0;
/// ublksrv_loop vs. /dev/loop
pub const UBLKSRV_TGT_TYPE_LOOP: c_uint = 1;
pub const UBLKSRV_TGT_TYPE_MAX: c_uint = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_tgt_info {
    pub dev_size: c_ulonglong,
    /// at most in-flight ios
    pub tgt_ring_depth: c_uint,
    pub nr_fds: c_uint,
    pub fds: [c_int; UBLKSRV_TGT_MAX_FDS as usize],
    pub tgt_data: *mut c_void,
    pub ops: *const ublksrv_tgt_type,
}
d!(ublksrv_tgt_info);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_tgt_type {
    /// One IO request comes from /dev/ublkbN, so notify target code
    /// for handling the IO. Inside target code, the IO can be handled
    /// with our io_uring too, if this is true, ->tgt_io_done callback
    /// has to be implemented. Otherwise, target can implement
    /// ->handle_event() for processing io completion there.
    pub handle_io_async:
        Option<unsafe extern "C" fn(arg1: *mut ublksrv_queue, tag: c_int) -> c_int>,

    /// target io is handled by our io_uring, and once the target io
    /// is completed, this callback is called
    pub tgt_io_done:
        Option<unsafe extern "C" fn(arg1: *mut ublksrv_queue, arg2: *mut iouring::io_uring_cqe)>,

    /// Someone has written to our eventfd, so let target handle the
    /// event, most of times, it is for handling io completion by
    /// calling ublksrv_complete_io() which has to be run in ubq_daemon context.
    ///
    /// Follows the typical scenario:
    ///
    /// 1) one target io is completed in target pthread context, so
    /// target code calls ublksrv_queue_send_event for notifying ubq daemon
    ///
    /// 2) ubq daemon gets notified, so wakeup from io_uring_enter(),
    /// then found eventfd is completed, so call ->handle_event()
    ///
    /// 3) inside ->handle_event(), if any io represented by one io
    /// command is completed, ublksrv_complete_io() is called for this io.
    ///
    /// 4) after returning from ->handle_event(), ubq_daemon will
    /// queue & submit the eventfd io immediately for getting
    /// notification from future event.
    pub handle_event: Option<unsafe extern "C" fn(arg1: *mut ublksrv_queue)>,

    /// One typical use case is to flush meta data, which is usually done
    /// in background. So there isn't any tag from libublksrv for this kind
    /// of IOs, and the target code has to request for allocating extra ios
    /// by passing tgt_type->extra_ios and let this callback consume & handle
    /// these extra IOs.
    ///
    /// @nr_queued_io: count of queued IOs in ublksrv_reap_events_uring of this time
    pub handle_io_background:
        Option<unsafe extern "C" fn(arg1: *mut ublksrv_queue, nr_queued_io: c_int)>,

    /// show target specific command line for adding new device
    ///
    /// Be careful: this callback is the only one which is not run from
    /// ublk device daemon task context.
    pub usage_for_add: Option<unsafe extern "C" fn()>,

    /// initialize this new target, argc/argv includes target specific
    /// command line parameters
    pub init_tgt: Option<
        unsafe extern "C" fn(
            arg1: *mut ublksrv_dev,
            type_: c_int,
            argc: c_int,
            argv: *mut *mut c_char,
        ) -> c_int,
    >,

    /// deinitialize this target
    pub deinit_tgt: Option<unsafe extern "C" fn(arg1: *mut ublksrv_dev)>,

    pub alloc_io_buf:
        Option<unsafe extern "C" fn(q: *mut ublksrv_queue, tag: c_int, size: c_int) -> *mut c_void>,
    pub free_io_buf:
        Option<unsafe extern "C" fn(q: *mut ublksrv_queue, buf: *mut c_void, tag: c_int)>,

    pub type_: c_int,

    /// flags required for ublk driver
    pub ublk_flags: c_uint,
    /// flags required for ublksrv
    pub ublksrv_flags: c_uint,
    /// extra io slots allocated for handling
    pub extra_ios: c_int,
    /// target specific IOs, such as meta io
    pub name: *const c_char,
}
d!(ublksrv_tgt_type);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ublksrv_dev {
    pub tgt: ublksrv_tgt_info,
    pub __queues: [*mut ublksrv_queue; MAX_NR_HW_QUEUES as usize],
    pub io_buf_start: *mut c_char,
    pub thread: *mut libc::pthread_t,
    pub cdev_fd: c_int,
    pub pid_file_fd: c_int,
    pub ctrl_dev: *const ublksrv_ctrl_dev,
    pub target_data: *mut c_void,
}
d!(ublksrv_dev);

pub unsafe fn ublksrv_get_iod(q: *const ublksrv_queue, tag: c_int) -> *mut cmd::ublksrv_io_desc {
    let idx = tag as usize * std::mem::size_of::<cmd::ublksrv_io_desc>();
    (*q).io_cmd_buf.add(idx) as *mut cmd::ublksrv_io_desc
}

pub unsafe fn build_user_data(
    tag: c_uint,
    op: c_uint,
    tgt_data: c_uint,
    is_target_io: c_uint,
) -> __u64 {
    assert!((tag >> 16 == 0) && (op >> 8 == 0) && (tgt_data >> 16 == 0));
    tag as __u64 | (op << 16) as __u64 | (tgt_data << 24) as __u64 | (is_target_io as __u64) << 63
}

pub unsafe fn user_data_to_tag(user_data: __u64) -> c_int {
    (user_data & 0xffff) as c_int
}

pub unsafe fn user_data_to_op(user_data: __u64) -> c_int {
    ((user_data >> 16) & 0xff) as c_int
}

pub unsafe fn user_data_to_tgt_data(user_data: __u64) -> c_int {
    ((user_data >> 24) & 0xffff) as c_int
}

extern "C" {
    pub fn ublksrv_ctrl_deinit(dev: *mut ublksrv_ctrl_dev);
    pub fn ublksrv_ctrl_init(data: *mut ublksrv_dev_data) -> *mut ublksrv_ctrl_dev;
    pub fn ublksrv_ctrl_get_affinity(ctrl_dev: *mut ublksrv_ctrl_dev) -> c_int;
    pub fn ublksrv_ctrl_add_dev(dev: *mut ublksrv_ctrl_dev) -> c_int;
    pub fn ublksrv_ctrl_del_dev(dev: *mut ublksrv_ctrl_dev) -> c_int;
    pub fn ublksrv_ctrl_get_info(dev: *mut ublksrv_ctrl_dev) -> c_int;
    pub fn ublksrv_ctrl_stop_dev(dev: *mut ublksrv_ctrl_dev) -> c_int;
    pub fn ublksrv_ctrl_dump(dev: *mut ublksrv_ctrl_dev, buf: *const c_char);
    pub fn ublksrv_ctrl_start_dev(ctrl_dev: *mut ublksrv_ctrl_dev, daemon_pid: c_int) -> c_int;
    pub fn ublksrv_ctrl_set_params(
        dev: *mut ublksrv_ctrl_dev,
        params: *mut cmd::ublk_params,
    ) -> c_int;
    pub fn ublksrv_ctrl_get_params(
        dev: *mut ublksrv_ctrl_dev,
        params: *mut cmd::ublk_params,
    ) -> c_int;
    pub fn ublksrv_dev_init(ctrl_dev: *const ublksrv_ctrl_dev) -> *mut ublksrv_dev;
    pub fn ublksrv_dev_deinit(dev: *mut ublksrv_dev);
}

/// target json has to include the following key/value
pub const UBLKSRV_TGT_NAME_MAX_LEN: u32 = 32;
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublksrv_tgt_base_json {
    pub name: [c_char; UBLKSRV_TGT_NAME_MAX_LEN as usize],
    pub type_: c_int,
    pub dev_size: c_ulonglong,
}

extern "C" {
    pub fn ublksrv_json_write_dev_info(
        dev: *const ublksrv_ctrl_dev,
        buf: *mut c_char,
        len: c_int,
    ) -> c_int;
    pub fn ublksrv_json_read_dev_info(
        json_buf: *const c_char,
        info: *mut cmd::ublksrv_ctrl_dev_info,
    ) -> c_int;
    pub fn ublksrv_json_write_queue_info(
        dev: *const ublksrv_ctrl_dev,
        jbuf: *mut c_char,
        len: c_int,
        qid: c_int,
        ubq_daemon_tid: c_int,
    ) -> c_int;
    pub fn ublksrv_json_read_queue_info(
        jbuf: *const c_char,
        qid: c_int,
        tid: *mut c_uint,
        affinity_buf: *mut c_char,
        len: c_int,
    ) -> c_int;
    pub fn ublksrv_json_read_target_info(
        jbuf: *const c_char,
        tgt_buf: *mut c_char,
        len: c_int,
    ) -> c_int;
    pub fn ublksrv_json_write_target_str_info(
        jbuf: *mut c_char,
        len: c_int,
        name: *const c_char,
        val: *const c_char,
    ) -> c_int;
    pub fn ublksrv_json_write_target_long_info(
        jbuf: *mut c_char,
        len: c_int,
        name: *const c_char,
        val: c_long,
    ) -> c_int;
    pub fn ublksrv_json_write_target_ulong_info(
        jbuf: *mut c_char,
        len: c_int,
        name: *const c_char,
        val: c_ulong,
    ) -> c_int;
    pub fn ublksrv_json_dump(jbuf: *const c_char);
    pub fn ublksrv_json_read_target_base_info(
        jbuf: *const c_char,
        tgt: *mut ublksrv_tgt_base_json,
    ) -> c_int;
    pub fn ublksrv_json_write_target_base_info(
        jbuf: *mut c_char,
        len: c_int,
        tgt: *const ublksrv_tgt_base_json,
    ) -> c_int;
    pub fn ublksrv_json_read_params(p: *mut cmd::ublk_params, jbuf: *const c_char) -> c_int;
    pub fn ublksrv_json_write_params(
        p: *const cmd::ublk_params,
        jbuf: *mut c_char,
        len: c_int,
    ) -> c_int;
    pub fn ublksrv_json_dump_params(jbuf: *const c_char) -> c_int;
    pub fn ublksrv_json_get_length(jbuf: *const c_char) -> c_int;
}

pub unsafe fn ublksrv_queue_get_data(q: *const ublksrv_queue) -> *mut c_void {
    (*q).private_data
}

extern "C" {
    pub fn ublksrv_queue_init(
        dev: *mut ublksrv_dev,
        q_id: c_ushort,
        nr_extra_ios: c_int,
        queue_data: *mut c_void,
    ) -> *mut ublksrv_queue;
    pub fn ublksrv_queue_deinit(q: *mut ublksrv_queue);
    pub fn ublksrv_queue_handled_event(q: *mut ublksrv_queue) -> c_int;
    pub fn ublksrv_queue_send_event(q: *mut ublksrv_queue) -> c_int;
    pub fn ublksrv_get_queue(dev: *const ublksrv_dev, q_id: c_int) -> *mut ublksrv_queue;
    pub fn ublksrv_process_io(q: *mut ublksrv_queue) -> c_int;
    pub fn ublksrv_complete_io(q: *mut ublksrv_queue, tag: c_uint, res: c_int) -> c_int;
    pub fn ublksrv_register_tgt_type(type_: *mut ublksrv_tgt_type) -> c_int;
    pub fn ublksrv_unregister_tgt_type(type_: *mut ublksrv_tgt_type);
    pub fn ublksrv_for_each_tgt_type(
        handle_tgt_type: Option<
            unsafe extern "C" fn(idx: c_uint, type_: *const ublksrv_tgt_type, data: *mut c_void),
        >,
        data: *mut c_void,
    );
    pub fn ublksrv_find_tgt_type(name: *const c_char) -> *const ublksrv_tgt_type;
    pub fn ublksrv_apply_oom_protection();
}

/* --------------------------------------------*/

/* --------------------------------------------*/
