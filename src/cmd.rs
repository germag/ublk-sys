// SPDX-License-Identifier: MIT
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)] // FIXME
use libc::{__s32, __u16, __u32, __u64, __u8};

/// Admin commands, issued by ublk server, and handled by ublk driver.
pub const UBLK_CMD_GET_QUEUE_AFFINITY: u32 = 1;
pub const UBLK_CMD_GET_DEV_INFO: u32 = 2;
pub const UBLK_CMD_ADD_DEV: u32 = 4;
pub const UBLK_CMD_DEL_DEV: u32 = 5;
pub const UBLK_CMD_START_DEV: u32 = 6;
pub const UBLK_CMD_STOP_DEV: u32 = 7;
pub const UBLK_CMD_SET_PARAMS: u32 = 8;
pub const UBLK_CMD_GET_PARAMS: u32 = 9;

/// IO commands, issued by ublk server, and handled by ublk driver.
///
/// FETCH_REQ: issued via sqe(URING_CMD) beforehand for fetching IO request
///      from ublk driver, should be issued only when starting device. After
///      the associated cqe is returned, request's tag can be retrieved via
///      cqe->userdata.
///
/// COMMIT_AND_FETCH_REQ: issued via sqe(URING_CMD) after ublkserver handled
///      this IO request, request's handling result is committed to ublk
///      driver, meantime FETCH_REQ is piggyback, and FETCH_REQ has to be
///      handled before completing io request.
///
/// NEED_GET_DATA: only used for write requests to set io addr and copy data
///      When NEED_GET_DATA is set, ublksrv has to issue UBLK_IO_NEED_GET_DATA
///      command after ublk driver returns UBLK_IO_RES_NEED_GET_DATA.
///
///      It is only used if ublksrv set UBLK_F_NEED_GET_DATA flag while starting a ublk device.
pub const UBLK_IO_FETCH_REQ: u32 = 32;
pub const UBLK_IO_COMMIT_AND_FETCH_REQ: u32 = 33;
pub const UBLK_IO_NEED_GET_DATA: u32 = 34;

/// only ABORT means that no re-fetch
pub const UBLK_IO_RES_OK: u32 = 0;
pub const UBLK_IO_RES_NEED_GET_DATA: u32 = 1;
pub const UBLK_IO_RES_ABORT: i32 = -libc::ENODEV;

pub const UBLKSRV_CMD_BUF_OFFSET: u32 = 0;
pub const UBLKSRV_IO_BUF_OFFSET: u32 = 0x80000000;

/// tag bit is 12bit, so at most 4096 IOs for each queue
pub const UBLK_MAX_QUEUE_DEPTH: u32 = 4096;

/// zero copy requires 4k block size, and can remap ublk driver's io
/// request into ublksrv's vm space
pub const UBLK_F_SUPPORT_ZERO_COPY: u32 = 1 << 0;

/// Force to complete io cmd via io_uring_cmd_complete_in_task so that
/// performance comparison is done easily with using task_work_add
pub const UBLK_F_URING_CMD_COMP_IN_TASK: u32 = 1 << 1;

/// User should issue io cmd again for write requests to
/// set io buffer address and copy data from bio vectors
/// to the userspace io buffer.
///
/// In this mode, task_work is not used.
pub const UBLK_F_NEED_GET_DATA: u32 = 1 << 2;

/// device state
pub const UBLK_S_DEV_DEAD: u32 = 0;
pub const UBLK_S_DEV_LIVE: u32 = 1;

/// shipped via sqe->cmd of io_uring command
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublksrv_ctrl_cmd {
    /// sent to which device, must be valid
    pub dev_id: __u32,
    /// sent to which queue, must be -1 if the cmd isn't for queue
    pub queue_id: __u16,
    /// cmd specific buffer, can be IN or OUT
    pub len: __u16,
    pub addr: __u64,
    /// inline data
    pub data: [__u64; 2],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublksrv_ctrl_dev_info {
    pub nr_hw_queues: __u16,
    pub queue_depth: __u16,
    pub state: __u16,
    pub pad0: __u16,
    pub max_io_buf_bytes: __u32,
    pub dev_id: __u32,
    pub ublksrv_pid: __s32,
    pub pad1: __u32,
    pub flags: __u64,
    /// For ublksrv internal use, invisible to ublk driver
    pub ublksrv_flags: __u64,
    pub reserved0: __u64,
    pub reserved1: __u64,
    pub reserved2: __u64,
}

pub const UBLK_IO_OP_READ: u32 = 0;
pub const UBLK_IO_OP_WRITE: u32 = 1;
pub const UBLK_IO_OP_FLUSH: u32 = 2;
pub const UBLK_IO_OP_DISCARD: u32 = 3;
pub const UBLK_IO_OP_WRITE_SAME: u32 = 4;
pub const UBLK_IO_OP_WRITE_ZEROES: u32 = 5;

pub const UBLK_IO_F_FAILFAST_DEV: u32 = 1 << 8;
pub const UBLK_IO_F_FAILFAST_TRANSPORT: u32 = 1 << 9;
pub const UBLK_IO_F_FAILFAST_DRIVER: u32 = 1 << 10;
pub const UBLK_IO_F_META: u32 = 1 << 11;
pub const UBLK_IO_F_FUA: u32 = 1 << 13;
pub const UBLK_IO_F_NOUNMAP: u32 = 1 << 15;
pub const UBLK_IO_F_SWAP: u32 = 1 << 16;

/// io cmd is described by this structure, and stored in share memory, indexed by request tag.
///
/// The data is stored by ublk driver, and read by ublksrv after one fetch command returns.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublksrv_io_desc {
    /// op: bit 0-7, flags: bit 8-31
    pub op_flags: __u32,
    pub nr_sectors: __u32,
    /// start sector for this io
    pub start_sector: __u64,
    /// buffer address in ublksrv daemon vm space, from ublk driver
    pub addr: __u64,
}

pub unsafe fn ublksrv_get_op(iod: *const ublksrv_io_desc) -> __u8 {
    ((*iod).op_flags & 0xff) as __u8
}

pub unsafe fn ublksrv_get_flags(iod: *const ublksrv_io_desc) -> __u32 {
    (*iod).op_flags >> 8
}

/// issued to ublk driver via /dev/ublkcN
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublksrv_io_cmd {
    pub q_id: __u16,
    /// for fetch/commit which result
    pub tag: __u16,
    /// io result, it is valid for COMMIT* command only
    pub result: __s32,
    /// userspace buffer address in ublksrv daemon process, valid for * FETCH* command only
    pub addr: __u64,
}

pub const UBLK_ATTR_READ_ONLY: u32 = 1 << 0;
pub const UBLK_ATTR_ROTATIONAL: u32 = 1 << 1;
pub const UBLK_ATTR_VOLATILE_CACHE: u32 = 1 << 2;
pub const UBLK_ATTR_FUA: u32 = 1 << 3;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublk_param_basic {
    pub attrs: __u32,
    pub logical_bs_shift: __u8,
    pub physical_bs_shift: __u8,
    pub io_opt_shift: __u8,
    pub io_min_shift: __u8,
    pub max_sectors: __u32,
    pub chunk_sectors: __u32,
    pub dev_sectors: __u64,
    pub virt_boundary_mask: __u64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublk_param_discard {
    pub discard_alignment: __u32,
    pub discard_granularity: __u32,
    pub max_discard_sectors: __u32,
    pub max_write_zeroes_sectors: __u32,
    pub max_discard_segments: __u16,
    pub reserved0: __u16,
}

pub const UBLK_PARAM_TYPE_BASIC: u32 = 1 << 0;
pub const UBLK_PARAM_TYPE_DISCARD: u32 = 1 << 1;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ublk_params {
    /// Total length of parameters, userspace has to set 'len' for both
    /// SET_PARAMS and GET_PARAMS command, and driver may update len
    /// if two sides use different version of 'ublk_params', same with
    /// 'types' fields.
    pub len: __u32,
    pub types: __u32,
    pub basic: ublk_param_basic,
    pub discard: ublk_param_discard,
}
