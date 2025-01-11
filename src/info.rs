use core::{convert::From, ffi::CStr, mem, time};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    os::{fd::AsRawFd, unix::ffi::OsStrExt},
    path::PathBuf,
};

use btrfs_sys::{btrfs_ioctl_get_subvol_info_args, BTRFS_IOCTL_MAGIC};

use crate::item::Uuid;

nix::ioctl_read!(
    btrfs_get_subvol_info,
    BTRFS_IOCTL_MAGIC,
    60,
    btrfs_ioctl_get_subvol_info_args
);

#[derive(Clone, Debug)]
pub struct SubvolInfo {
    tree_id: u64,
    name: PathBuf,
    parent_id: u64,
    dirid: u64,
    generation: u64,
    flags: u64,
    uuid: Uuid,
    parent_uuid: Uuid,
    received_uuid: Uuid,
    ctransid: u64,
    otransid: u64,
    stransid: u64,
    rtransid: u64,
    ctime: time::Duration,
    otime: time::Duration,
    stime: time::Duration,
    rtime: time::Duration,
}

impl SubvolInfo {
    pub(crate) fn from_c_struct(info: btrfs_ioctl_get_subvol_info_args) -> Self {
        Self {
            tree_id: info.treeid,
            parent_id: info.parent_id,
            name: {
                let cstr = unsafe { &CStr::from_ptr(info.name.as_slice().as_ptr()) };
                let osstr = <OsStr as OsStrExt>::from_bytes(cstr.to_bytes());
                PathBuf::from(osstr)
            },
            dirid: info.dirid,
            generation: info.generation,
            flags: info.flags,
            uuid: Uuid(info.uuid),
            parent_uuid: Uuid(info.parent_uuid),
            received_uuid: Uuid(info.received_uuid),
            ctransid: info.ctransid,
            otransid: info.otransid,
            stransid: info.stransid,
            rtransid: info.rtransid,
            ctime: time::Duration::from_secs(info.ctime.sec)
                + time::Duration::from_nanos(info.ctime.nsec as u64),
            otime: time::Duration::from_secs(info.otime.sec)
                + time::Duration::from_nanos(info.otime.nsec as u64),
            stime: time::Duration::from_secs(info.stime.sec)
                + time::Duration::from_nanos(info.stime.nsec as u64),
            rtime: time::Duration::from_secs(info.rtime.sec)
                + time::Duration::from_nanos(info.rtime.nsec as u64),
        }
    }
}

pub fn subvol_info(file: &File) -> nix::Result<SubvolInfo> {
    let mut args: btrfs_ioctl_get_subvol_info_args = unsafe { mem::zeroed() };

    unsafe { btrfs_get_subvol_info(file.as_raw_fd(), &mut args as *mut _)? };

    Ok(SubvolInfo::from_c_struct(args))
}
